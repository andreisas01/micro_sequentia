use super::processor::signals;

struct Memory {
    data: Vec<u16>,
}

struct Controller;

pub struct MemoryInterface {
    memory: Memory,
}

pub fn load_program(program_data: &[u8]) -> MemoryInterface {
    let mut mem_interf = MemoryInterface::new();

    for (i, chunk) in program_data.chunks(2).enumerate() {
        let value = if chunk.len() == 2 {
            u16::from_be_bytes([chunk[0], chunk[1]])
        } else {
            (chunk[0] as u16) << 8
        };
        mem_interf.memory.data[i] = value;
    }

    mem_interf
}

impl MemoryInterface {
    fn new() -> Self {
        Self {
            memory: Memory { data: vec![0; 65536] },
        }
    }

    pub fn process_signal(&mut self, signal: &signals::Memory, IR: &mut u16, ADR: &mut u16, MDR: &mut u16) {
        use signals::Memory::*;

        match signal {
            None => {},
            IFCH => *IR = Controller::read(&self.memory, *ADR),
            READ => *MDR = Controller::read(&self.memory, *ADR),
            WRITE => Controller::write(&mut self.memory, *ADR, *MDR),
        }
    }

}

// local memory command device (DCLM)
impl Controller {
    fn read(memory: &Memory, address: u16) -> u16 {
        memory.data[address as usize]
    }

    fn write(memory: &mut Memory, address: u16, value: u16) {
        memory.data[address as usize] = value;
    }
}