use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn run_fuzzer(data: &[u8], counter: usize) {
    //println!("Start Fuzzing {}",counter);
    // let cmd=Command::new("/home/v/exif/exif")
    //     .arg("mutated.jpg")
    //     .stdin(Stdio::null())
    //     .stdout(Stdio::null())
    //     .stderr(Stdio::null())
    //     .spawn();
    let output = Command::new("/home/v/fuzzer/rfuzz/exif")
        .arg("mutated.jpg")
        .output()
        .unwrap();
    if !output.status.success() {
        let file_path = format!("crashes/crash.{}.jpg", counter);
        let mut file = match File::create(&file_path) {
            Ok(file) => file,
            Err(_) => return,
        };
        if let Err(_) = file.write_all(data) {
            return;
        }
    }
    // let pid=match cmd{
    //     Ok(child)=>Pid::from_raw(child.id() as i32),
    //     Err(_)=>return,
    // };

    // match waitpid(Some(pid),Some(WaitPidFlag::WNOHANG)){
    //     Ok(WaitStatus::Stopped(_,signal::Signal::SIGSEGV))=>{
    // let file_path=format!("crash.{}.jpg",counter);
    // let mut file=match File::create(&file_path){
    //     Ok(file)=>file,
    //     Err(_)=>return,
    // };
    // if let Err(_)=file.write_all(data){
    //     return;
    // }
    //     }
    //     _=>return,
    // }
}
