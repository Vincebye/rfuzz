use nix::sys::ptrace;
use nix::sys::wait::{wait, waitpid, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{exit, Command, Stdio};

extern crate linux_personality;
use linux_personality::personality;
use nix::sys::signal::Signal;
use std::ffi::c_void;
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
fn handle_sigstop(pid: Pid, saved_values: &HashMap<u64, i64>) {
    let mut regs = ptrace::getregs(pid).unwrap();
    println!("Hit breakpoint at 0x{:x}", regs.rip - 1);
    match saved_values.get(&(regs.rip - 1)) {
        Some(orig) => {
            restore_breakpoint(pid, regs.rip - 1, *orig);

            // rewind rip
            regs.rip -= 1;
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
pub fn run_child(bp_map: &Vec<u64>, bp_mapping: &mut HashMap<u64, i64>) -> Pid {
    // Execute binary spawning new process
    let child = unsafe {
        Command::new("/home/v/fuzzer/rfuzz/exif")
            .arg("/home/v/fuzzer/rfuzz/1.jpg")
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

            for bp in bp_map {
                bp_mapping.insert(base + bp, set_breakpoint(res, base + bp));
            }
            ptrace::cont(res, None).expect("Should have continued");
        }
        _ => println!("COULD NOT START"),
    }
    res
}
// Code that runs only for parent
pub fn run_parent(pid: Pid, breakpoints: &Vec<u64>, bp_mapping: &HashMap<u64, i64>) {
    //cal converage
    let all_map_count = breakpoints.capacity();
    let mut hit_count = 0;

    let mut _trace: Vec<u64> = Vec::new();

    loop {
        println!("{}/{}", hit_count, all_map_count);
        match waitpid(pid, None) {
            Ok(WaitStatus::Stopped(pid_t, sig_num)) => match sig_num {
                Signal::SIGTRAP => {
                    hit_count += 1;
                    handle_sigstop(pid_t, &bp_mapping);
                }

                Signal::SIGSEGV => {
                    let regs = ptrace::getregs(pid_t).unwrap();
                    println!("Segmentation fault at 0x{:x}", regs.rip);
                    break;
                }
                _ => {
                    println!("Some other signal - {}", sig_num);
                    break;
                }
            },

            Ok(WaitStatus::Exited(pid, exit_status)) => {
                // println!(
                //     "Process with pid: {} exited with status {}",
                //     pid, exit_status
                // );
                break;
            }

            Ok(status) => {
                println!("Received status: {:?}", status);
                ptrace::cont(pid, None).expect("Failed to deliver signal");
            }

            Err(err) => {
                println!("Some kind of error - {:?}", err);
            }
        }
    }
}
// fn main() {
//     // breakpoints to set
//     let breakpoints: [u64; 1] = [0x5555555553b1];
//     match unsafe{fork()} {
//         Ok(ForkResult::Child) => {
//             run_child();
//         }
//         Ok(ForkResult::Parent {child}) => {
//             run_parent(child, &breakpoints);
//         }

//         Err(err) => {
//             panic!("[main] fork() failed: {}", err);
//         }
//     };
// }
