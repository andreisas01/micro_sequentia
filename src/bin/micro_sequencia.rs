#![expect(non_snake_case, unused_crate_dependencies)]

use micro_sequentia::*;

fn main() {
    let MPM = microprogram::get_microprogram();
    for micro_instruction in &MPM {
        println!("{micro_instruction}");
    }
}