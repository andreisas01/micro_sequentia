#![expect(unused_variables, clippy::case_sensitive_file_extension_comparisons, clippy::cast_possible_truncation)]

mod isa;
mod parser;
mod encoder;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 || !args[1].ends_with(".asm") {
        println!("Usage: {} <input_file>.asm", args[0].split(std::path::MAIN_SEPARATOR).next_back().unwrap());
        return;
    }

    let input_file = &args[1];

    let inputs_result = std::fs::read_to_string(input_file);
    let inputs = match inputs_result {
        Err(e) => {
            println!("Failed to read the input file ({input_file}).");
            println!("Error: {e}");
            return;
        }
        Ok(content) => content,
    };

    let parsed_code = match parser::parse_text(&inputs) {
        Ok(parsed) => parsed,
        Err(e) => {
            println!("Failed to parse assembly.");
            println!("Error: {e}");
            return;
        }
    };

    let bit_table = encoder::BitTable::generate();

    std::fs::write(format!("{}.obj", input_file.trim_end_matches(".asm")),
    encoder::encode_parsed_code(parsed_code, &bit_table)).unwrap();
}
