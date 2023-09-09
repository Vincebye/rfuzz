use rand::prelude::*;
pub fn mutate(mut data:Vec<u8>)->Vec<u8>{
    let flip_ratio:f32=0.2;
    let flips=((data.len()-4) as f32*flip_ratio) as usize;
    let mut rng=thread_rng();
    let flip_indexes:Vec<usize>=(0..flips)
    .map(|_| rng.gen_range(2..(data.len()-6)))
    .collect();

    //let methods=[0,1];
    let methods=[0];
    for &idx in flip_indexes.iter(){
        let method=methods.choose(&mut rng).unwrap();
        if *method==0{
            data[idx]=bit_flip(data[idx]);
        }
        else{
            magic(&mut data,idx);
        }

    }
    data
}
fn bit_flip(byte: u8) -> u8 {
    let flip_mask = [1, 2, 4, 8, 16, 32, 64, 128];
    let mut rng=thread_rng();
    let flip_value = flip_mask.choose(&mut rng).unwrap();
    byte ^ flip_value
}

fn magic(data:&mut Vec<u8>,idx:usize){
    unimplemented!();
}