use clap::ArgMatches;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct RuntimeConcig {
    pub exec_name: String,
    pub corpus: Vec<Vec<u8>>,
    pub bpmap: Vec<u64>,
}

impl RuntimeConcig {
    pub fn new(matches: ArgMatches) -> Self {
        let target = matches.get_one::<String>("target").unwrap();
        let corpus = matches.get_one::<String>("corpus").unwrap();
        let bpmap = matches.get_one::<String>("bpmap").unwrap();

        RuntimeConcig {
            exec_name: target.clone(),
            corpus: read_corpus(Path::new(&corpus)),
            bpmap: read_bpmap(Path::new(&bpmap)),
        }
    }
}
pub fn read_corpus(path: &Path) -> Vec<Vec<u8>> {
    let mut files: Vec<Vec<u8>> = vec![];

    if path.is_dir() {
        for file in fs::read_dir(path).unwrap() {
            let data = fs::read(file.unwrap().path()).unwrap();
            files.push(data);
        }
    } else {
        files.push(fs::read(path).unwrap());
    }
    files
}

pub fn read_bpmap(path: &Path) -> Vec<u64> {
    let mut bpmap: Vec<u64> = vec![];
    if path.is_file() {
        let fh = fs::File::open(path).unwrap();
        let reader = BufReader::new(fh);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if !line.trim().is_empty() {
                        bpmap.push(line.parse::<u64>().unwrap());
                    }
                }
                Err(_) => println!("Filed to read bpmap"),
            }
        }
    };
    bpmap
}
