// the window. drag stuff in (or browse for it), pick where it goes, hit decode.
// kept the actual xor/file logic in lib.rs so i can test it without
// needing a display to be up.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use dexor::{collect_jobs, run_job, DEFAULT_XOR_KEY};
use eframe::egui;

const AUTHOR: &str = "Fuzzdkk";
const AUTHOR_URL: &str = "https://github.com/Fuzzdkk";

// vendors whose quarantine is a plain whole-file single-byte xor — these are
// the ones dexor decodes directly. picking one just fills in the key box.
// (others like eset/mcafee-bup/symantec/kaspersky use byte-transforms,
// containers or rc4, so a single xor won't fully decode them.)
const PRESETS: &[(&str, u8)] = &[
    ("Cisco AMP / Secure Endpoint", 0x77),
    ("Microsoft MSE / Antimalware", 0xFF),
    ("SentinelOne", 0xFF),
    ("Microsoft Defender (macOS)", 0x25),
    ("VIPRE", 0x33),
];

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 480.0])
            .with_min_inner_size([420.0, 340.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "DeXOR",
        options,
        Box::new(|_cc| Box::<App>::default()),
    )
}

struct App {
    /// Files/folders queued but not yet processed.
    pending: Vec<PathBuf>,
    /// Single-byte XOR key.
    key: u8,
    log: Vec<String>,
    ok_count: usize,
    err_count: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
            key: DEFAULT_XOR_KEY,
            log: Vec::new(),
            ok_count: 0,
            err_count: 0,
        }
    }
}

impl App {
    fn decode(&mut self) {
        if self.pending.is_empty() {
            self.log.push("⚠ Nothing queued yet.".into());
            return;
        }

        let jobs = match collect_jobs(&self.pending) {
            Ok(j) => j,
            Err(e) => {
                self.log.push(format!("✗ Could not read inputs: {e}"));
                return;
            }
        };
        self.log
            .push(format!("→ Decoding {} file(s) with key 0x{:02X}…", jobs.len(), self.key));

        for job in &jobs {
            match run_job(job, self.key) {
                Ok(dest) => {
                    self.ok_count += 1;
                    self.log.push(format!("✓ {}", dest.display()));
                }
                Err(e) => {
                    self.err_count += 1;
                    self.log
                        .push(format!("✗ {} — {e}", job.source.display()));
                }
            }
        }
        self.log.push(format!(
            "Done: {} decoded, {} failed.",
            self.ok_count, self.err_count
        ));
        self.pending.clear();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Pull in anything dropped onto the window this frame.
        ctx.input(|i| {
            for f in &i.raw.dropped_files {
                if let Some(path) = &f.path {
                    if !self.pending.contains(path) {
                        self.pending.push(path.clone());
                    }
                }
            }
        });

        // little credit bar pinned to the bottom.
        egui::TopBottomPanel::bottom("credit").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.weak(format!("made by {AUTHOR} ·"));
                ui.hyperlink_to(AUTHOR_URL, AUTHOR_URL);
            });
            ui.add_space(2.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("DeXOR");
            ui.label(
                "XOR-decode files. Drag files or folders onto the window (or use \
                 the browse buttons), set the key, and hit Decode.",
            );
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Output goes into a new ");
                ui.monospace(dexor::OUTPUT_DIR_NAME);
                ui.label(" folder next to each input, each file named ");
                ui.monospace(format!("{}<original>", dexor::FILENAME_PREFIX));
                ui.label(".");
            });
            ui.separator();

            // XOR key.
            ui.horizontal(|ui| {
                ui.label("XOR key:");
                let mut hex = format!("{:02X}", self.key);
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut hex)
                        .desired_width(40.0)
                        .char_limit(2),
                );
                if resp.changed() {
                    if let Ok(v) = u8::from_str_radix(hex.trim(), 16) {
                        self.key = v;
                    }
                }
                ui.weak("(single byte, hex)");

                // vendor presets — selecting one just sets the key above.
                let current = PRESETS
                    .iter()
                    .find(|(_, k)| *k == self.key)
                    .map(|(name, _)| *name)
                    .unwrap_or("Custom");
                egui::ComboBox::from_id_source("preset")
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        for (name, key) in PRESETS {
                            if ui
                                .selectable_label(self.key == *key, format!("{name}  (0x{key:02X})"))
                                .clicked()
                            {
                                self.key = *key;
                            }
                        }
                    });
            });

            ui.separator();

            // browse buttons — a choice next to drag-and-drop, and the reliable
            // path on wayland where dnd can be flaky.
            ui.horizontal(|ui| {
                if ui.button("➕ Add files…").clicked() {
                    if let Some(files) = rfd::FileDialog::new().pick_files() {
                        for f in files {
                            if !self.pending.contains(&f) {
                                self.pending.push(f);
                            }
                        }
                    }
                }
                if ui.button("➕ Add folder…").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        if !self.pending.contains(&dir) {
                            self.pending.push(dir);
                        }
                    }
                }
            });

            // Drop zone / pending list.
            egui::Frame::none()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::Margin::same(12.0))
                .rounding(6.0)
                .show(ui, |ui| {
                    ui.set_min_height(90.0);
                    ui.set_width(ui.available_width());
                    if self.pending.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(24.0);
                            ui.weak("⬇  Drop files/folders here, or use the buttons above");
                        });
                    } else {
                        ui.label(format!("{} item(s) queued:", self.pending.len()));
                        for p in &self.pending {
                            ui.monospace(p.display().to_string());
                        }
                    }
                });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let can_run = !self.pending.is_empty();
                if ui
                    .add_enabled(can_run, egui::Button::new("▶ Decode"))
                    .clicked()
                {
                    self.decode();
                }
                if ui.button("Clear queue").clicked() {
                    self.pending.clear();
                }
                if ui.button("Clear log").clicked() {
                    self.log.clear();
                    self.ok_count = 0;
                    self.err_count = 0;
                }
            });

            ui.separator();
            ui.label("Log:");
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.log {
                        ui.monospace(line);
                    }
                });
        });

        // Visual feedback while a drag is hovering over the window.
        if ctx.input(|i| !i.raw.hovered_files.is_empty()) {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("drop_overlay"),
            ));
            let screen = ctx.screen_rect();
            painter.rect_filled(
                screen,
                0.0,
                egui::Color32::from_black_alpha(160),
            );
            painter.text(
                screen.center(),
                egui::Align2::CENTER_CENTER,
                "Drop to queue",
                egui::FontId::proportional(28.0),
                egui::Color32::WHITE,
            );
        }
    }
}
