use clap::{Arg, Command};
use mutate::mutate;
use nix::unistd::{fork, ForkResult, Pid};
use std::path::Path;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufReader, Read, Write},
    time::Instant,
};
mod execute;
mod forkserver;
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

fn read_to_vec(filepath: &str) -> io::Result<Vec<u64>> {
    let mut bp_vec = Vec::new();
    let bpfile = Path::new(filepath);
    if bpfile.is_file() {
        let file = File::open(filepath).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(parse_num) = line?.parse::<u64>() {
                bp_vec.push(parse_num);
            } else {
                println!("Filed to parse line as u64");
            }
        }
        println!("{:?}", bp_vec);
        Ok(bp_vec)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The bpfile is not exists!",
        ))
    }
}
#[derive(Default)]
struct FuzzingStats {
    crash_count: u64,
    execute_count: u64,
    max_coverage: f64,
}

fn main() -> io::Result<()> {
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

    let bpfile_path = "/home/v/fuzzer/rfuzz/data/breakpoints.map";

    let mut stats = FuzzingStats::default();
    let bp_map = read_to_vec(bpfile_path)?;
    let _bp_count = bp_map.capacity();
    println!("{}", _bp_count);
    let mut bp_mapping: HashMap<u64, i64> = HashMap::new();
    loop {
        let child = forkserver::run_child(&bp_map, &mut bp_mapping);
        stats.execute_count += 1;
        forkserver::run_parent(child, &bp_map, &bp_mapping);
    }
    // let mut file = File::open("1.jpg").expect("filed to open the file");
    // let mut buffer = Vec::new();
    // file.read_to_end(&mut buffer).expect("Failed yo read file");
    Ok(())

    // let start_time = Instant::now();
    // let mut last_exec_per_sec = 0.0;
    // for i in 1..1100 {
    //     let elapsed_time = start_time.elapsed();
    //     let exec_per_sec = i as f64 / elapsed_time.as_secs_f64();
    //     last_exec_per_sec = exec_per_sec;
    //     print!(" -> {:.0} exec/sec\r", exec_per_sec);
    //     std::io::stdout().flush().unwrap();
    //     let data = buffer.clone();
    //     let mutated_data = mutate(data);
    //     write_to_file(&mutated_data);
    //     execute::run_fuzzer(&mutated_data, i);
    // }
    // println!("-> {:.0} exec/sec", last_exec_per_sec);
}
