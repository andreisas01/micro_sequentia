// micro_sequencia binary crate

#![expect(unused_crate_dependencies, clippy::case_sensitive_file_extension_comparisons)]

use micro_sequentia::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 || !args[1].ends_with(".obj") {
        println!("Usage: {} <input_file>.obj", args[0].split(std::path::MAIN_SEPARATOR).next_back().unwrap());
        return;
    }

    let input_file = &args[1];

    let program_data_result = std::fs::read(input_file);
    let program_data = match program_data_result {
        Err(e) => {
            println!("Failed to read the input file ({input_file}).");
            println!("Error: {e}");
            return;
        }
        Ok(data) => data,
    };

    let ram = memory::load_program(&program_data);
    let mut cpu = processor::CPU::new(ram);
    cpu.run();
}