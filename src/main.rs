use std::{fs::{File, self}, io::{Read, Write}};

use clap::{Arg, Command};
use mutate::mutate;
mod mutate;
mod execute;

fn write_to_file(data:&[u8])->std::io::Result<()>{
    if let Err(e)=fs::remove_file("mutated.jpg"){
        if e.kind()!=std::io::ErrorKind::NotFound{
            return Err(e);
        }
    }
    let mut file=File::create("mutated.jpg")?;
    file.write_all(data)?;
    Ok(())
}

fn main() {
    let _matches=Command::new("Rfuzz")
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("TARGET")
                .required(false)
                .help("The target to be fuzzed")
        )
        .get_matches();
    let mut file=File::open("1.jpg").expect("filed to open the file");
    let mut buffer=Vec::new();
    file.read_to_end(&mut buffer).expect("Failed yo read file");

    for i in 1..1100{
        //println!("Start the cycle {}",i);
        let data=buffer.clone();
        let mutated_data=mutate(data);
        write_to_file(&mutated_data);
        execute::run_fuzzer(&mutated_data,i);
    }
   
}
