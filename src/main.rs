use std::{
    fs::{self, File},
    io::{Read, Write},
    time::Instant,
};
use clap::{Arg, Command};
use std::path::Path;
use mutate::mutate;
mod execute;
mod mutate;

fn write_to_file(data: &[u8]) -> std::io::Result<()> {
    if let Err(e) = fs::remove_file("mutated.jpg") {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(e);
        }
    }
    let mut file = File::create("mutated.jpg")?;
    file.write_all(data)?;
    Ok(())
}

fn main() {
    let _matches = Command::new("Rfuzz")
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("TARGET")
                .required(false)
                .help("The target to be fuzzed"),
        )
        .get_matches();

    let bpfile_path="/home/v/fuzzer/rfuzz/data/breakpoints.map";
    if bpfile_path.is
    let mut file = File::open("1.jpg").expect("filed to open the file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed yo read file");
    

    let start_time = Instant::now();
    let mut last_exec_per_sec = 0.0;
    for i in 1..1100 {
        let elapsed_time = start_time.elapsed();
        let exec_per_sec = i as f64 / elapsed_time.as_secs_f64();
        last_exec_per_sec = exec_per_sec;
        print!(" -> {:.0} exec/sec\r", exec_per_sec);
        std::io::stdout().flush().unwrap();
        let data = buffer.clone();
        let mutated_data = mutate(data);
        write_to_file(&mutated_data);
        execute::run_fuzzer(&mutated_data, i);
    }
    println!("-> {:.0} exec/sec", last_exec_per_sec);
}
