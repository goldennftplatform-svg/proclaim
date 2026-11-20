use std::{env, fs};

use risc0_binfmt::ProgramBinary;
use risc0_zkos_v1compat::V1COMPAT_ELF;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = &args[1];
    let output = &args[2];
    let user_elf = fs::read(input).expect("read user ELF");
    let binary = ProgramBinary::new(&user_elf, V1COMPAT_ELF);
    let blob = binary.encode();
    fs::write(output, &blob).expect("write .bin");
    println!("wrote {} bytes to {}", blob.len(), output);
}
