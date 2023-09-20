use rand::prelude::*;
use std::collections::BTreeSet;
use std::fs;
#[derive(Debug, Clone)]
enum MutationMethod {
    Raw,
    BitFlip,
}

pub struct Mutator {
    corpus: Vec<Sample>,
    samples: Vec<Sample>,
    trace_list: BTreeSet<Vec<u64>>,
    rng: rand::prelude::ThreadRng,
}

impl Mutator {
    pub fn new() -> Self {
        Mutator {
            corpus: Vec::new(),
            samples: Vec::new(),
            trace_list: BTreeSet::new(),
            rng: thread_rng(),
        }
    }

    pub fn consume(&mut self, corpus: &Vec<Vec<u8>>) {
        for entry in corpus {
            self.samples.push(Sample::new(entry));
        }
    }

    pub fn update(&mut self, samples: &Vec<Sample>) {
        for sample in samples {
            match &sample.trace {
                Some(trace) => {
                    if !self.trace_list.contains(trace) {
                        // println!(
                        //     "[-]New coverage for input {:?} [{:?}]",
                        //     sample.data, sample.method
                        // );
                        self.trace_list.insert(trace.clone());
                        self.corpus.push(sample.clone());
                    }
                }
                None => {
                    println!("[!]No trace info.......");
                }
            }
        }
        self.mutate();
    }
    fn mutate(&mut self) {
        for sample in &mut self.corpus {
            for _ in 0..100 {
                &self.samples.push(sample.mutate(&mut self.rng));
            }
        }
    }
}
impl Iterator for Mutator {
    type Item = Sample;
    fn next(&mut self) -> Option<Self::Item> {
        self.samples.pop()
    }
}

#[derive(Debug, Clone)]
pub struct Sample {
    data: Vec<u8>,
    method: MutationMethod,
    trace: Option<Vec<u64>>,
}
impl Sample {
    fn new(data: &Vec<u8>) -> Self {
        Sample {
            data: data.clone(),
            method: MutationMethod::Raw,
            trace: None,
        }
    }

    fn mutate(&mut self, rng: &mut ThreadRng) -> Sample {
        let strategy = rng.gen_range(0..=1);
        match strategy {
            0 => self.bit_flip(rng),
            1 => self.bit_flip(rng),
            _ => self.raw(),
        }
    }

    pub fn add_trace(&mut self, trace: Vec<u64>) {
        self.trace = Some(trace);
    }

    pub fn materialize_sample(&self, filename: &str) {
        fs::write(filename, &self.data).expect("Failed to materialize sample!");
    }

    fn raw(&mut self) -> Sample {
        Sample {
            data: self.data.clone(),
            method: MutationMethod::Raw,
            trace: None,
        }
    }

    fn bit_flip(&mut self, rng: &mut ThreadRng) -> Sample {
        let mut bytecode = self.data.to_vec();
        let flip_mask = [1, 2, 4, 8, 16, 32, 64, 128];

        let flip_ratio: f32 = 0.2;
        let flips = ((bytecode.len() - 4) as f32 * flip_ratio) as usize;
        let flip_indexes: Vec<usize> = (0..flips)
            .map(|_| rng.gen_range(2..(bytecode.len() - 6)))
            .collect();

        for &idx in flip_indexes.iter() {
            let flip_value = flip_mask.choose(rng).unwrap();
            bytecode[idx] = bytecode[idx] ^ flip_value;
        }
        Sample {
            data: bytecode,
            method: MutationMethod::BitFlip,
            trace: None,
        }
    }
}

