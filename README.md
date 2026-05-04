# micro_sequentia

`micro_sequentia` is a Rust-based microarchitecture playground with two main parts:

- an assembler that turns custom `.asm` source into raw `.obj` machine code
- a CPU simulator that can run the generated object code in a headless CLI or a visual GUI

The project models a word-oriented machine with 16-bit registers, a 4 KiB memory space, a microprogrammed control unit, and an assembly language with labels, multiple addressing modes, and branch instructions.

## Features

- Assemble source files into `.obj` binaries.
- Run assembled programs from the command line.
- Inspect execution live in a GUI with register, bus, memory, and microinstruction views.
- Load source directly in the GUI, assemble it in place, and step or run execution.
- Highlight the currently executing source line while a program is being simulated.

## Requirements

- Rust toolchain with Cargo
- A desktop environment for the GUI binary

## Project Layout

- `src/lib.rs` exposes the assembler, memory system, and CPU simulator core.
- `src/assembler/` contains parsing and encoding logic for assembly programs.
- `src/processor/` contains the ISA model, signals, and microprogrammed control unit.
- `src/bin/assembler_cli.rs` assembles `.asm` files into `.obj` files.
- `src/bin/processor_cli.rs` loads `.obj` files and runs them headlessly.
- `src/bin/processor_gui.rs` launches the visual CPU simulator.
- `fibbonacci.asm` is an example program.

## Build And Run

Build everything:

```bash
cargo build
```

Check the workspace without producing binaries:

```bash
cargo check
```

Run the assembler:

```bash
cargo run --bin assembler_cli -- path/to/program.asm
```

This reads the `.asm` file and writes `path/to/program.obj` next to it.

Run the headless processor:

```bash
cargo run --bin processor_cli -- path/to/program.obj
```

Launch the GUI simulator:

```bash
cargo run --bin processor_gui
```

For a quick smoke test of the GUI build, use:

```bash
cargo check --bin processor_gui
```

## Assembly Workflow

1. Write a program in the custom assembly language.
2. Assemble it into an object file with `assembler_cli`, or paste it into the GUI and use `ASSEMBLE & LOAD`.
3. Load the resulting `.obj` file into `processor_cli` or `processor_gui`.
4. Run, step, or inspect state in the GUI.

## Assembly Language Overview

The assembler accepts decimal immediates, register operands, indirect operands, indexed operands, labels, and inline comments.

Supported register syntax:

- `R0` through `R15`

Supported operand forms:

- immediate: `24`
- direct register: `R3`
- indirect register: `(R3)`
- indexed register: `256(R4)`

Comments are stripped when they start with `;` or `#`.
Labels can appear on their own line or before an instruction, for example `loop:` or `main: mov R0, 1`.

The parser also normalizes the multi-word opcodes `PUSH PC`, `POP PC`, `PUSH FLAG`, and `POP FLAG`.

### Instruction Groups

`B1` instructions take two operands:

- `MOV`
- `ADD`
- `SUB`
- `CMP`
- `AND`
- `OR`
- `XOR`

`B2` instructions take one operand:

- `CLR`
- `NEG`
- `INC`
- `DEC`
- `ASL`
- `ASR`
- `LSR`
- `ROL`
- `ROR`
- `RLC`
- `RRC`
- `JMP`
- `CALL`
- `PUSH`
- `POP`

`B3` instructions are relative branches:

- `BR`
- `BNE`
- `BEQ`
- `BPL`
- `BMI`
- `BCS`
- `BCC`
- `BVS`
- `BVC`

`B4` instructions are control and system operations:

- `CLC`
- `CLV`
- `CLZ`
- `CLS`
- `CCC`
- `SEC`
- `SEV`
- `SEZ`
- `SES`
- `SCC`
- `NOP`
- `RET`
- `RETI`
- `HALT`
- `WAIT`
- `PUSH_PC`
- `POP_PC`
- `PUSH_FLAG`
- `POP_FLAG`

## Encoding Notes

- Object files are raw little-endian 16-bit words written as bytes.
- B1 and B2 instructions may emit extra words for immediate and indexed operands.
- Branches are encoded as PC-relative offsets resolved from labels.
- The processor loads program bytes into word-addressed memory starting at address `0x0000`.

## GUI Notes

The GUI is built with `eframe` and includes:

- source editor with syntax-aware line highlighting during execution
- buttons for loading source, assembling, loading binaries, running, pausing, stepping, micro-stepping, and resetting
- live readouts for registers, buses, flags, memory interface state, and the microinstruction register
- a memory inspector window with a hexadecimal byte view
- a MIR signal snapshot popup that decodes the current control signals

The editor is locked while a compiled binary is loaded, so you must reset the CPU to return to source editing.

## Example

The repository includes `fibbonacci.asm`, which computes a Fibonacci value and stores the result at memory address `0x0100`.

```bash
cargo run --bin assembler_cli -- fibbonacci.asm
cargo run --bin processor_cli -- fibbonacci.obj
```

## Notes For Development

- The workspace is configured with strict Rust and Clippy lints.
- The GUI binary intentionally keeps `#![expect(unused_crate_dependencies)]` because of the way the workspace is organized.
- The CPU simulator API is centered around `processor::CPU::tick()`, `snapshot()`, and `microprogram()`.
