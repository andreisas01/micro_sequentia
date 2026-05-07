// processor binary crate

#![expect(unused_crate_dependencies, clippy::case_sensitive_file_extension_comparisons)]

use micro_sequentia::*;

const DEFAULT_BINARY_NAME: &str = "processor_cli";
const DEFAULT_FREQUENCY_HZ: u32 = 100;

fn print_usage(binary_name: &str) {
    println!("Usage: {binary_name} <input_file>.obj [frequency_hz]");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let binary_name = args[0].split(std::path::MAIN_SEPARATOR).next_back().unwrap_or(DEFAULT_BINARY_NAME);

    if !(args.len() == 2 || args.len() == 3) || !args[1].ends_with(".obj") {
        print_usage(binary_name);
        return;
    }

    let input_file = &args[1];
    let frequency_hz = if args.len() == 3 {
        match args[2].parse::<u32>() {
            Ok(0) => {
                println!("Frequency must be a positive integer.");
                print_usage(binary_name);
                return;
            }
            Ok(value) => value,
            Err(error) => {
                println!("Invalid frequency value ({}).", args[2]);
                println!("Error: {error}");
                print_usage(binary_name);
                return;
            }
        }
    } else {
        DEFAULT_FREQUENCY_HZ
    };

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
    cpu.run(frequency_hz);
}