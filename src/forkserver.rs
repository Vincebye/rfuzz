use nix::sys::ptrace;
use nix::sys::wait::{wait, waitpid, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{exit, Command, Stdio};

extern crate linux_personality;
use linux_personality::personality;
use nix::sys::signal::Signal;
use std::ffi::c_void;

use crate::config;

pub enum ParentStatus {
    Finished(Vec<u64>),
    Crash(u64),
}

fn set_breakpoint(pid: Pid, addr: u64) -> i64 {
    // Read 8 bytes from the process memory
    let value = ptrace::read(pid, (addr) as *mut c_void).unwrap() as u64;

    // Insert breakpoint by write new values
    let bp = (value & (u64::MAX ^ 0xFF)) | 0xCC;

    unsafe {
        ptrace::write(pid, addr as *mut c_void, bp as *mut c_void).unwrap();
    }

    // Return original bytecode
    value as i64
}
fn restore_breakpoint(pid: Pid, addr: u64, orig_value: i64) {
    unsafe {
        // Restore original bytecode
        ptrace::write(pid, addr as *mut c_void, orig_value as *mut c_void).unwrap();
    }
}
fn handle_sigstop(pid: Pid, saved_values: &HashMap<u64, i64>, trace: &mut Vec<u64>,hit_breakpoints:&mut HashSet<u64>) {
    let mut regs = ptrace::getregs(pid).unwrap();
    println!("Hit breakpoint at 0x{:x}", regs.rip - 1);
    hit_breakpoints.insert(regs.rip - 1);
    match saved_values.get(&(regs.rip - 1)) {
        Some(orig) => {
            restore_breakpoint(pid, regs.rip - 1, *orig);
            // rewind rip
            regs.rip -= 1;
            trace.push(regs.rip);
            ptrace::setregs(pid, regs).expect("Error rewinding RIP");
        }
        _ => print!("Nothing saved here"),
    }

    ptrace::cont(pid, None).expect("Restoring breakpoint failed");
}
fn get_executable_base(filename: String) -> Option<u64> {
    let filename = Path::new(&filename);

    if filename.is_file() {
        let fh = fs::File::open(filename).unwrap();
        let reader = BufReader::new(fh);

        for line in reader.lines() {
            let line = line.unwrap();
            let fields: Vec<&str> = line.split_whitespace().collect();

            if fields[1].contains("x") {
                let addr: Vec<&str> = fields[0].split("-").collect();
                let base: u64 =
                    u64::from_str_radix(addr[0], 16).expect("Failed parsing base address");
                return Some(base);
            }
        }
        None
    } else {
        None
    }
}
// Code that runs only for child
pub fn run_child(
    config: &config::RuntimeConcig,
    bp_mapping: &mut HashMap<u64, i64>,
    filename: &str,
) -> Pid {
    // Execute binary spawning new process
    let child = unsafe {
        Command::new(&config.exec_name)
            .arg(filename)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .pre_exec(|| {
                ptrace::traceme().expect("Process doesn't want to be traced ...");
                personality(linux_personality::ADDR_NO_RANDOMIZE).unwrap();
                Ok(())
            })
            .spawn()
            .expect("[!] Failed to run process")
    };
    let res = Pid::from_raw(child.id() as i32);
    match waitpid(res, None) {
        Ok(WaitStatus::Stopped(_, Signal::SIGTRAP)) => {
            // Get file base
            let base = get_executable_base(format!("/proc/{}/maps", res))
                .expect("[!] Failed to get executable base!");

            for bp in &config.bpmap {
                bp_mapping.insert(base + bp, set_breakpoint(res, base + bp));
            }
            ptrace::cont(res, None).expect("Should have continued");
        }
        _ => println!("COULD NOT START"),
    }
    res
}
// Code that runs only for parent
pub fn run_parent(
    pid: Pid,
    bp_mapping: &HashMap<u64, i64>,
    hit_breakpoints:&mut HashSet<u64>
) -> ParentStatus {
    //cal converage

    let mut trace: Vec<u64> = Vec::new();

    loop {
        match waitpid(pid, None) {
            Ok(WaitStatus::Stopped(pid_t, sig_num)) => match sig_num {
                Signal::SIGTRAP => {
                    handle_sigstop(pid_t, &bp_mapping, &mut trace,hit_breakpoints);
                }

                Signal::SIGSEGV => {
                    let regs = ptrace::getregs(pid_t).unwrap();
                    println!("Segmentation fault at 0x{:x}", regs.rip);
                    return ParentStatus::Crash(regs.rip);
                }
                _ => {
                    println!("Some other signal - {}", sig_num);
                    break;
                }
            },

            Ok(WaitStatus::Exited(_, _)) => return ParentStatus::Finished(trace),

            Ok(status) => {
                println!("Received status: {:?}", status);
                ptrace::cont(pid, None).expect("Failed to deliver signal");
            }

            Err(err) => {
                println!("Some kind of error - {:?}", err);
            }
        }
    }

    ParentStatus::Finished(trace)
}
