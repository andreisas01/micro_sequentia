#![expect(unused_crate_dependencies, clippy::needless_range_loop)]

use eframe::egui;
use micro_sequentia::*;
use rfd::FileDialog;
use std::collections::HashMap;
use std::fmt::Write as _;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        "micro sequentia - CPU Simulator",
        options,
        Box::new(|_cc| {
            Ok(Box::<CPUSimulator>::default())
        }),
    )
}

struct CPUSimulator {
    cpu: processor::CPU,
    is_running: bool,
    ticks_per_second: u16,
    tick_accumulator: f32,
    asm_editor_code: String,
    loaded_binary_path: String,
    loaded_from_binary_file: bool,
    show_memory_inspector: bool,
    show_mir_signals_popup: bool,
    address_to_source_line: HashMap<u16, usize>,
    highlighted_source_line: Option<usize>,
    previous_snapshot: Option<processor::CpuSnapshot>,
    status_message: String,
    status_is_error: bool,
}

impl Default for CPUSimulator {
    fn default() -> Self {
        Self {
            cpu: processor::CPU::new(memory::MemoryInterface::new()),
            is_running: false,
            ticks_per_second: 60,
            tick_accumulator: 0.0,
            asm_editor_code: "; Write assembly code here\nhalt\n".to_owned(),
            loaded_binary_path: String::new(),
            loaded_from_binary_file: false,
            show_memory_inspector: false,
            show_mir_signals_popup: false,
            address_to_source_line: HashMap::new(),
            highlighted_source_line: None,
            previous_snapshot: None,
            status_message: "No program loaded".to_owned(),
            status_is_error: false,
        }
    }
}

impl CPUSimulator {
    fn refresh_highlighted_line(&mut self, snapshot: &processor::CpuSnapshot) {
        let mapped_line = self.address_to_source_line
            .get(&snapshot.adr)
            .copied();

        if let Some(line) = mapped_line {
            self.highlighted_source_line = Some(line);
        }
    }

    fn set_status(&mut self, is_error: bool, message: impl Into<String>) {
        self.status_message = message.into();
        self.status_is_error = is_error;
    }

    fn assemble_editor_into_processor(&mut self) {
        let source = self.asm_editor_code.as_str();
        if source.trim().is_empty() {
            self.set_status(true, "Assembly editor is empty");
            return;
        }

        let parsed = assembler::parser::parse_text(source)
            .map_err(|error| format!("Assembly parse error: {error}"));
        let parsed = match parsed {
            Ok(parsed) => parsed,
            Err(error) => {
                self.set_status(true, error);
                return;
            }
        };

        self.address_to_source_line.clear();
        for instruction in &parsed.instructions {
            self.address_to_source_line
                .insert(instruction.address, instruction.source_line);
        }

        let bit_table = assembler::encoder::OpcodeTable::generate();
        let obj_data = assembler::encoder::encode_parsed_code(parsed, &bit_table);
        let byte_count = obj_data.len();
        self.replace_cpu_from_bytes(&obj_data);
        let snapshot = self.cpu.snapshot();
        self.refresh_highlighted_line(&snapshot);
        self.loaded_binary_path.clear();
        self.loaded_binary_path.push_str("[assembled from editor]");
        self.loaded_from_binary_file = false;
        self.set_status(false, format!("Assembled and loaded {byte_count} bytes"));
    }

    fn replace_cpu_from_bytes(&mut self, program_data: &[u8]) {
        let ram = memory::load_program(program_data);
        self.cpu = processor::CPU::new(ram);
        self.is_running = false;
        self.tick_accumulator = 0.0;
        self.previous_snapshot = None;
        self.show_mir_signals_popup = false;
    }

    fn load_binary_from_picker(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Object files", &["obj"])
            .pick_file()
        {
            let path_str = path.to_string_lossy().to_string();

            match std::fs::read(&path_str) {
                Ok(program_data) => {
                    let byte_count = program_data.len();
                    self.replace_cpu_from_bytes(&program_data);
                    self.address_to_source_line.clear();
                    self.highlighted_source_line = None;
                    self.loaded_binary_path.clear();
                    self.loaded_binary_path.push_str(&path_str);
                    self.loaded_from_binary_file = true;
                    self.set_status(false, format!("Loaded {byte_count} bytes from {path_str}"));
                }
                Err(error) => {
                    self.set_status(true, format!("Failed to load {path_str}: {error}"));
                }
            }
        }
    }

    fn load_source_from_picker(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("Assembly files", &["asm"])
            .pick_file()
        {
            let path_str = path.to_string_lossy().to_string();

            match std::fs::read_to_string(&path_str) {
                Ok(source) => {
                    self.asm_editor_code.clear();
                    self.asm_editor_code.push_str(&source);
                    self.highlighted_source_line = None;
                    self.loaded_from_binary_file = false;
                    self.set_status(false, format!("Loaded source from {path_str}"));
                }
                Err(error) => {
                    self.set_status(true, format!("Failed to load {path_str}: {error}"));
                }
            }
        }
    }

    fn run_cpu(&mut self) {
        if self.cpu.is_halted() {
            self.set_status(true, "CPU is halted. Load binary or assemble first.");
            return;
        }

        self.is_running = true;
        self.tick_accumulator = 0.0;
        self.set_status(false, "Running machine code");
    }

    fn pause_cpu(&mut self) {
        self.is_running = false;
        self.tick_accumulator = 0.0;
        self.set_status(false, "Execution paused");
    }

    fn reset_cpu(&mut self) {
        self.cpu = processor::CPU::new(memory::MemoryInterface::new());
        self.is_running = false;
        self.tick_accumulator = 0.0;
        self.loaded_binary_path.clear();
        self.address_to_source_line.clear();
        self.highlighted_source_line = None;
        self.loaded_from_binary_file = false;
        self.previous_snapshot = None;
        self.show_mir_signals_popup = false;
        self.set_status(false, "CPU reset with empty memory");
    }
}

impl eframe::App for CPUSimulator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.is_running {
            let dt = ctx.input(|input| input.stable_dt.max(0.0));
            self.tick_accumulator += f32::from(self.ticks_per_second) * dt;

            while self.tick_accumulator >= 1.0 {
                if self.cpu.is_halted() {
                    self.is_running = false;
                    break;
                }
                self.previous_snapshot = Some(self.cpu.snapshot());
                self.cpu.tick();
                self.tick_accumulator -= 1.0;
            }

            ctx.request_repaint();
        }

        egui::SidePanel::left("control_and_editor_panel")
            .resizable(true)
            .default_width(560.0)
            .show(ctx, |ui| {
            ui.heading("Program Control");
            let big_button_size = egui::vec2(150.0, 36.0);
            let has_loaded_program = !self.loaded_binary_path.is_empty();
            let editor_locked = self.is_running || !self.loaded_binary_path.is_empty();

            ui.horizontal(|ui| {
                if ui.add_enabled(
                    !editor_locked,
                    egui::Button::new("OPEN SOURCE").min_size(big_button_size),
                ).clicked() {
                    self.load_source_from_picker();
                }
                if ui.add_sized(big_button_size, egui::Button::new("ASSEMBLE & LOAD")).clicked() {
                    self.assemble_editor_into_processor();
                }
                ui.separator();
                if ui.add_sized(big_button_size, egui::Button::new("LOAD BINARY")).clicked() {
                    self.load_binary_from_picker();
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.add_enabled(
                    self.is_running || has_loaded_program,
                    egui::Button::new(if self.is_running { "PAUSE" } else { "RUN" })
                        .min_size(big_button_size),
                ).clicked() {
                    if self.is_running {
                        self.pause_cpu();
                    } else {
                        self.run_cpu();
                    }
                }
                ui.separator();
                if ui.add_enabled(
                    has_loaded_program,
                    egui::Button::new("micro STEP").min_size(big_button_size),
                ).clicked() {
                    self.is_running = false;
                    self.previous_snapshot = Some(self.cpu.snapshot());
                    self.cpu.tick();
                }
                if ui.add_enabled(
                    has_loaded_program,
                    egui::Button::new("STEP").min_size(big_button_size),
                ).clicked() {
                    self.is_running = false;
                    self.previous_snapshot = Some(self.cpu.snapshot());
                    while !self.cpu.is_halted() {
                        self.cpu.tick();
                        let snapshot = self.cpu.snapshot();
                        if snapshot.ir != self.previous_snapshot.as_ref().unwrap().ir {
                            break;
                        }
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.label("CPU Frequency");
                ui.add(
                    egui::Slider::new(&mut self.ticks_per_second, 1..=2000),
                );
                ui.label("Hz");

                ui.add_space(55.5);
                if ui.add_enabled(
                    has_loaded_program,
                    egui::Button::new("RESET").min_size(big_button_size),
                ).clicked() {
                    self.reset_cpu();
                }
            });

            if !self.loaded_binary_path.is_empty() {
                ui.label(format!("Loaded binary: {}", self.loaded_binary_path));
            }

            if self.status_is_error {
                ui.colored_label(egui::Color32::LIGHT_RED, self.status_message.as_str());
            } else {
                ui.colored_label(egui::Color32::LIGHT_GREEN, self.status_message.as_str());
            }

            if editor_locked {
                ui.colored_label(
                    egui::Color32::KHAKI,
                    "ASM editor is locked while a binary is loaded (use RESET to unlock)",
                );
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.heading("ASM Editor");
                if self.loaded_from_binary_file {
                    ui.colored_label(
                        egui::Color32::LIGHT_GRAY,
                        egui::RichText::new("BINARY LOADED").strong(),
                    );
                }
            });
            if self.loaded_from_binary_file {
                ui.add_space(8.0);
                ui.colored_label(
                    egui::Color32::GRAY,
                    "Source view is hidden because a compiled .obj binary is loaded.",
                );
                ui.colored_label(
                    egui::Color32::GRAY,
                    "Press RESET to unlock the editor, then load/open source if needed.",
                );
            } else {
                let snapshot = self.cpu.snapshot();
                self.refresh_highlighted_line(&snapshot);
                let highlighted_source_line = self.highlighted_source_line;

                let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                    let mut layout_job = egui::text::LayoutJob::default();
                    layout_job.wrap.max_width = wrap_width;

                    let normal_format = egui::TextFormat {
                        font_id: egui::FontId::monospace(15.0),
                        color: ui.visuals().text_color(),
                        ..Default::default()
                    };
                    let highlighted_format = egui::TextFormat {
                        color: egui::Color32::from_rgb(255, 230, 120),
                        ..normal_format.clone()
                    };

                    let lines: Vec<&str> = text.as_str().split('\n').collect();
                    for (index, line) in lines.iter().enumerate() {
                        let line_number = index + 1;
                        let format = if highlighted_source_line == Some(line_number) {
                            highlighted_format.clone()
                        } else {
                            normal_format.clone()
                        };
                        layout_job.append(line, 0.0, format);

                        if index + 1 < lines.len() {
                            layout_job.append("\n", 0.0, normal_format.clone());
                        }
                    }

                    ui.fonts_mut(|fonts| fonts.layout_job(layout_job))
                };

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.asm_editor_code)
                            .desired_rows(32)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace)
                            .layouter(&mut layouter)
                            .interactive(!editor_locked)
                            .code_editor(),
                    );
                });
            }
        });

        let snapshot = self.cpu.snapshot();
        let mir_signals = self.cpu.mir_signals_snapshot();
        let previous_snapshot = self.previous_snapshot.as_ref();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(egui::RichText::new("CPU Architecture").size(30.0).strong());
            ui.label(egui::RichText::new("Live State Overview").size(16.0));
            ui.separator();

            let bvi = (snapshot.flag & 0b1000_0000) != 0;
            let c = (snapshot.flag & 0b0000_1000) != 0;
            let z = (snapshot.flag & 0b0000_0100) != 0;
            let s = (snapshot.flag & 0b0000_0010) != 0;
            let v = (snapshot.flag & 0b0000_0001) != 0;

            let changed_color = egui::Color32::from_rgb(255, 230, 120);
            let value_text = |text: String, size: f32, changed: bool| {
                let mut rich = egui::RichText::new(text).size(size).monospace();
                if changed {
                    rich = rich.color(changed_color);
                }
                rich
            };

            let register_card = |ui: &mut egui::Ui, name: &str, value: String, changed: bool| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_min_width(88.0);
                    ui.set_min_height(41.0);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(name).size(17.0).strong());
                        ui.add_space(1.0);
                        ui.label(value_text(value, 15.0, changed));
                    });
                });
                ui.add_space(3.0);
            };

            let register_mem_card = |ui: &mut egui::Ui| -> bool {
                let mut clicked = false;

                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_min_width(88.0);
                    ui.set_min_height(41.0);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("MEM").size(17.0).strong());
                        if ui
                            .add_sized(egui::vec2(72.0, 20.0), egui::Button::new("Inspect"))
                            .clicked()
                        {
                            clicked = true;
                        }
                    });
                });

                ui.add_space(3.0);
                clicked
            };

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Memory Interface").size(11.0).italics());
            });
            ui.separator();
            let mut open_memory_from_card = false;
            ui.columns(3, |columns| {
                register_card(
                    &mut columns[0],
                    "ADR",
                    format!("0x{:04X}", snapshot.adr),
                    previous_snapshot.is_some_and(|prev| prev.adr != snapshot.adr),
                );
                if register_mem_card(&mut columns[1]) {
                    open_memory_from_card = true;
                }
                register_card(
                    &mut columns[2],
                    "MDR",
                    format!("0x{:04X}", snapshot.mdr),
                    previous_snapshot.is_some_and(|prev| prev.mdr != snapshot.mdr),
                );

            });

            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Buses").size(11.0).italics());
            });
            ui.separator();
            ui.columns(1, |columns| {
                register_card(
                    &mut columns[0],
                    "RBUS",
                    format!("0x{:04X}", snapshot.rbus),
                    previous_snapshot.is_some_and(|prev| prev.rbus != snapshot.rbus),
                );
            });
            ui.columns(2, |columns| {
                register_card(
                    &mut columns[0],
                    "SBUS",
                    format!("0x{:04X}", snapshot.sbus),
                    previous_snapshot.is_some_and(|prev| prev.sbus != snapshot.sbus),
                );
                register_card(
                    &mut columns[1],
                    "DBUS",
                    format!("0x{:04X}", snapshot.dbus),
                    previous_snapshot.is_some_and(|prev| prev.dbus != snapshot.dbus),
                );
            });

            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Registers").size(11.0).italics());
            });
            ui.separator();
            ui.columns(4, |columns| {
                register_card(
                    &mut columns[0],
                    "IR",
                    format!("0x{:04X}", snapshot.ir),
                    previous_snapshot.is_some_and(|prev| prev.ir != snapshot.ir),
                );
                register_card(
                    &mut columns[1],
                    "SP",
                    format!("0x{:04X}", snapshot.sp),
                    previous_snapshot.is_some_and(|prev| prev.sp != snapshot.sp),
                );
                register_card(
                    &mut columns[2],
                    "T",
                    format!("0x{:04X}", snapshot.t),
                    previous_snapshot.is_some_and(|prev| prev.t != snapshot.t),
                );
                register_card(
                    &mut columns[3],
                    "PC",
                    format!("0x{:04X}", snapshot.pc),
                    previous_snapshot.is_some_and(|prev| prev.pc != snapshot.pc),
                );
            });

            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Control Unit").size(11.0).italics());
            });
            ui.separator();
            ui.columns(2, |columns| {
                register_card(
                    &mut columns[0],
                    "MAR",
                    format!("0x{:04X}", snapshot.mar),
                    previous_snapshot.is_some_and(|prev| prev.mar != snapshot.mar),
                );

                let mir_changed = previous_snapshot.is_some_and(|prev| prev.mir != snapshot.mir);
                egui::Frame::group(columns[1].style()).show(&mut columns[1], |ui| {
                    ui.set_min_width(200.0);
                    ui.set_min_height(41.0);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("MIR").size(17.0).strong());
                        ui.add_space(1.0);
                        let mir_value_response = ui.add(
                            egui::Label::new(value_text(format!("0x{:09X}", snapshot.mir), 15.0, mir_changed))
                                .sense(egui::Sense::click()),
                        );
                        if mir_value_response.clicked() {
                            self.show_mir_signals_popup = true;
                        }
                    });
                });

                columns[1].add_space(3.0);
            });

            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Status").size(11.0).italics());
            });
            ui.separator();
            ui.columns(3, |columns| {
                register_card(
                    &mut columns[0],
                    "BPO",
                    u8::from(snapshot.bpo).to_string(),
                    previous_snapshot.is_some_and(|prev| prev.bpo != snapshot.bpo),
                );
                register_card(
                    &mut columns[1],
                    "FLAG",
                    format!(
                        "0x{:04X}\nBVI:{} C:{} Z:{} S:{} V:{}",
                        snapshot.flag,
                        u8::from(bvi),
                        u8::from(c),
                        u8::from(z),
                        u8::from(s),
                        u8::from(v)
                    ),
                    previous_snapshot.is_some_and(|prev| prev.flag != snapshot.flag),
                );
                register_card(
                    &mut columns[2],
                    "IVR",
                    format!("0x{:04X}", snapshot.ivr),
                    previous_snapshot.is_some_and(|prev| prev.ivr != snapshot.ivr),
                );
            });

            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("General Registers").size(11.0).italics());
            });
            ui.separator();
            ui.columns(8, |columns| {
                for i in 0..8 {
                    register_card(
                        &mut columns[i],
                        format!("R{i}").as_str(),
                        format!("0x{:04X}", snapshot.registers[i]),
                        previous_snapshot.is_some_and(|prev| prev.registers[i] != snapshot.registers[i]),
                    );
                }
                for i in 8..16 {
                    register_card(
                        &mut columns[i - 8],
                        format!("R{i}").as_str(),
                        format!("0x{:04X}", snapshot.registers[i]),
                        previous_snapshot.is_some_and(|prev| prev.registers[i] != snapshot.registers[i]),
                    );
                }
            });

            if open_memory_from_card {
                self.show_memory_inspector = true;
            }
        });

        if self.show_memory_inspector {
            let memory_bytes = self.cpu.isa.MEM.dump_bytes();
            let mut open = self.show_memory_inspector;

            egui::Window::new("Memory Inspector")
                .open(&mut open)
                .default_width(530.0)
                .default_height(460.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Hex View (16 bytes per line)").size(18.0).strong());
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (row_index, chunk) in memory_bytes.chunks(16).enumerate() {
                            let address = row_index * 16;
                            let mut bytes_text = String::new();
                            for byte in chunk {
                                let _ = write!(&mut bytes_text, " {byte:02X}");
                            }

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("{address:04X}:"))
                                        .size(16.0)
                                        .monospace()
                                        .strong(),
                                );
                                ui.label(egui::RichText::new(bytes_text).size(16.0).monospace());
                            });
                        }
                    });
                });

            self.show_memory_inspector = open;
        }

        if self.show_mir_signals_popup {
            let mut open = self.show_mir_signals_popup;

            egui::Window::new("MIR Signals Snapshot")
                .open(&mut open)
                .default_width(340.0)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(format!("MIR = 0x{:09X}", snapshot.mir))
                            .size(16.0)
                            .monospace()
                            .strong(),
                    );
                    ui.separator();

                    for line in [
                        format!("SBUS: {}", mir_signals.sbus),
                        format!("DBUS: {}", mir_signals.dbus),
                        format!("ALU: {}", mir_signals.alu),
                        format!("RBUS: {}", mir_signals.rbus),
                        format!("MEM: {}", mir_signals.memory),
                        format!("OTHER: {}", mir_signals.other),
                    ] {
                        ui.label(egui::RichText::new(line).size(14.0).monospace());
                    }
                });

            self.show_mir_signals_popup = open;
        }
    }
}