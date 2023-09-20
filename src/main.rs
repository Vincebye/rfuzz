use clap::{Arg, Command};
use nix::unistd::{fork, ForkResult, Pid};
use std::collections::HashSet;
use std::path::Path;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufReader, Read, Write},
    time::Instant,
};
mod config;
mod forkserver;
mod mutate;

const FILE: &str = "mutated.jpg";

#[derive(Default)]
struct FuzzingStats {
    crash_count: u64,
    execute_count: u64,
}

fn main() -> io::Result<()> {
    let matches = Command::new("Rfuzz")
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("TARGET")
                .required(false)
                .help("The target to be fuzzed"),
        )
        .arg(
            Arg::new("corpus")
                .short('c')
                .long("corpus")
                .value_name("CORPUS")
                .required(false)
                .help("File corpus consumed by the fuzzer"),
        )
        .arg(
            Arg::new("bpmap")
                .short('b')
                .long("bpmap")
                .value_name("BPMAP")
                .required(false)
                .help("The function breakmap list"),
        )
        .get_matches();
    //cargo run -- -target /home/v/fuzzer/rfuzz/exif -corpus /home/v/fuzzer/rfuzz/data/corpus -bpmap /home/v/fuzzer/rfuzz/data/breakpoints.map
    let runtime_config = config::RuntimeConcig::new(matches);
    //init the Mutator Engine
    let mut mutator = mutate::Mutator::new();

    mutator.consume(&runtime_config.corpus);
    let mut sample_pool: Vec<mutate::Sample> = vec![];
    let mut stats = FuzzingStats::default();
    let mut bp_mapping: HashMap<u64, i64> = HashMap::new();
    let mut hit_breakpoints: HashSet<u64> = HashSet::new();
    let start = Instant::now();
    let mut flag = true;
    while flag {
        for mut sample in &mut mutator {
            sample.materialize_sample(FILE);
            let child = forkserver::run_child(&runtime_config, &mut bp_mapping, FILE);
            stats.execute_count += 1;
            match forkserver::run_parent(child, &bp_mapping, &mut hit_breakpoints) {
                forkserver::ParentStatus::Finished(trace) => {
                    sample.add_trace(trace);
                    sample_pool.push(sample);
                }
                forkserver::ParentStatus::Crash(rip) => {
                    stats.crash_count += 1;
                    let crash_filename = format!("crash_{}", rip);
                    let _ = fs::copy(FILE, crash_filename);
                    flag = false;
                }
            }
        }

        mutator.update(&sample_pool);
    }
    let elapsed = start.elapsed().as_secs_f64();
    let hit_breakpoint = hit_breakpoints.capacity() as f64;
    let all_breakpoints = runtime_config.bpmap.capacity() as f64;
    println!("[{:10.2}] cases {:10} | speed  {:10.2} | crashes {:10} | HitBreakpoints {:10}] |Coverage Rate {:10.2}%",
            elapsed, stats.execute_count, (stats.execute_count as f64)/ elapsed, stats.crash_count,hit_breakpoints.capacity(),(hit_breakpoint/all_breakpoints)*100.0);

    Ok(())
}
