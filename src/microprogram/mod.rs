#[allow(clippy::missing_panics_doc)]
pub fn get_microprogram() -> Vec<u64> {
    // read from file, strip the 0x prefix, and save as u64 without converting to ascii
    let microprogram = std::fs::read_to_string("microprogram.txt").expect("Failed to read microprogram.txt.");
    microprogram
        .lines()
        .map(|line| line.trim_start_matches("0x").trim())
        .map(|line| u64::from_str_radix(line, 16).expect("Failed to parse microinstruction."))
        .collect()
}