use eframe::egui;
use rfd::FileDialog;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

pub fn run() -> Result<(), String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([700.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "acadrust DXF/DWG benchmark",
        options,
        Box::new(|_cc| Ok(Box::new(BenchApp::default()))),
    )
    .map_err(|error| error.to_string())
}

struct BenchApp {
    scale: String,
    iterations: String,
    files: Vec<PathBuf>,
    output: String,
    running: bool,
    receiver: Option<Receiver<RunResult>>,
    last_report: Option<PathBuf>,
    report_data: Option<GuiReport>,
    report_section: usize,
}

struct RunResult {
    output: String,
    report: PathBuf,
    report_data: Option<GuiReport>,
}

#[derive(Clone, Debug, Deserialize)]
struct GuiReport {
    sections: Vec<GuiSection>,
}

#[derive(Clone, Debug, Deserialize)]
struct GuiSection {
    name: String,
    results: Vec<GuiResult>,
}

#[derive(Clone, Debug, Deserialize)]
struct GuiResult {
    label: String,
    dxf_ms: Option<f64>,
    acadrust_ms: Option<f64>,
    acadsharp_ms: Option<f64>,
    ezdxf_ms: Option<f64>,
}

impl Default for BenchApp {
    fn default() -> Self {
        Self {
            scale: "large".into(),
            iterations: "5".into(),
            files: Vec::new(),
            output: "Choose a preset or add DXF/DXB/DWG files, then run the benchmark.".into(),
            running: false,
            receiver: None,
            last_report: None,
            report_data: None,
            report_section: 0,
        }
    }
}

impl eframe::App for BenchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(receiver) = &self.receiver {
            if let Ok(result) = receiver.try_recv() {
                self.output = result.output;
                self.running = false;
                self.last_report = Some(result.report);
                self.report_data = result.report_data;
                self.report_section = 0;
                self.receiver = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("acadrust DXF/DWG benchmark");
            ui.label("Run the generated comparison or benchmark your own CAD files.");
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Preset");
                    egui::ComboBox::from_id_salt("scale")
                        .selected_text(&self.scale)
                        .show_ui(ui, |ui| {
                            for scale in ["small", "medium", "large", "huge", "extrahuge"] {
                                ui.selectable_value(&mut self.scale, scale.into(), scale);
                            }
                        });
                    ui.label("Iterations");
                    ui.add(egui::TextEdit::singleline(&mut self.iterations).desired_width(60.0));
                });
                ui.horizontal(|ui| {
                    if ui.button("Add DXF / DXB / DWG files").clicked() && !self.running {
                        if let Some(files) = FileDialog::new()
                            .add_filter("CAD files", &["dxf", "dxb", "dwg"])
                            .pick_files()
                        {
                            self.files.extend(files);
                            self.files.sort();
                            self.files.dedup();
                        }
                    }
                    if ui.button("Clear files").clicked() && !self.running {
                        self.files.clear();
                    }
                });
                if self.files.is_empty() {
                    ui.label("No custom files selected. The preset run will be used.");
                } else {
                    ui.label(format!("{} custom file(s) selected:", self.files.len()));
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for file in &self.files {
                                ui.label(file.display().to_string());
                            }
                        });
                }
                ui.add_space(4.0);
                if self.running {
                    ui.add_enabled(false, egui::Button::new("Benchmark running..."));
                } else if ui.button("Run benchmark").clicked() {
                    self.start_run();
                }
            });

            ui.add_space(10.0);
            ui.heading("Library comparison");
            ui.horizontal_wrapped(|ui| {
                library_badge(ui, "dxf-rs", true);
                library_badge(ui, "acadrust", true);
                let acadsharp_ready = self
                    .report_data
                    .as_ref()
                    .map(|report| report_has_library(report, Library::AcadSharp))
                    .unwrap_or(false);
                let ezdxf_ready = self
                    .report_data
                    .as_ref()
                    .map(|report| report_has_library(report, Library::Ezdxf))
                    .unwrap_or(false);
                library_badge(ui, "ACadSharp (.NET)", acadsharp_ready);
                library_badge(ui, "ezdxf (Python)", ezdxf_ready);
            });
            if let Some(report) = &self.report_data {
                if !report.sections.is_empty() {
                    let selected = self.report_section.min(report.sections.len() - 1);
                    self.report_section = selected;
                    ui.horizontal(|ui| {
                        ui.label("Category");
                        egui::ComboBox::from_id_salt("report-section")
                            .selected_text(&report.sections[selected].name)
                            .show_ui(ui, |ui| {
                                for (index, section) in report.sections.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.report_section,
                                        index,
                                        &section.name,
                                    );
                                }
                            });
                    });
                    show_report_section(ui, &report.sections[selected]);
                }
            } else {
                ui.label("Run a benchmark to populate the comparison table.");
            }

            ui.add_space(10.0);
            ui.label("Run output");
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.output)
                            .font(egui::TextStyle::Monospace)
                            .desired_rows(22)
                            .desired_width(f32::INFINITY),
                    );
                });

            if let Some(report) = &self.last_report {
                ui.horizontal(|ui| {
                    ui.label(format!("Report: {}", report.display()));
                    if ui.button("Open Markdown report").clicked() {
                        open_file(report);
                    }
                    if ui.button("Open report folder").clicked() {
                        if let Some(parent) = report.parent() {
                            open_file(parent);
                        }
                    }
                });
            }
        });
    }
}

impl BenchApp {
    fn start_run(&mut self) {
        let iterations = match self.iterations.trim().parse::<usize>() {
            Ok(value) if value > 0 => value,
            _ => {
                self.output = "Iterations must be a positive integer.".into();
                return;
            }
        };

        let files = self.files.clone();
        let scale = self.scale.clone();
        let report = if files.is_empty() {
            PathBuf::from("bench_output")
                .join(scale_report_dir(&scale))
                .join("report.md")
        } else {
            PathBuf::from("bench_output/custom/report.md")
        };
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.running = true;
        self.last_report = None;
        self.report_data = None;
        self.report_section = 0;
        self.output = "Starting benchmark...".into();

        std::thread::spawn(move || {
            let executable = match std::env::current_exe() {
                Ok(path) => path,
                Err(error) => {
                    let _ = sender.send(RunResult {
                        output: error.to_string(),
                        report,
                        report_data: None,
                    });
                    return;
                }
            };
            let mut command = Command::new(executable);
            command.current_dir(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            command
                .arg("--iterations")
                .arg(iterations.to_string())
                .arg("--report")
                .arg(&report);
            if files.is_empty() {
                command.arg("--scale").arg(&scale);
            } else {
                for file in &files {
                    command.arg("--input").arg(file);
                }
            }

            let result = command.output();
            match result {
                Ok(output) => {
                    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.is_empty() {
                        text.push_str("\n\n--- stderr ---\n");
                        text.push_str(&stderr);
                    }
                    let json_path = report_for_json(&report);
                    let report_data = fs::read(&json_path)
                        .ok()
                        .and_then(|bytes| serde_json::from_slice::<GuiReport>(&bytes).ok());
                    let _ = sender.send(RunResult {
                        output: text,
                        report,
                        report_data,
                    });
                }
                Err(error) => {
                    let _ = sender.send(RunResult {
                        output: error.to_string(),
                        report,
                        report_data: None,
                    });
                }
            }
        });
    }
}

fn scale_report_dir(scale: &str) -> &'static str {
    match scale {
        "small" => "small_100",
        "medium" => "medium_1k",
        "huge" => "huge_100k",
        "extrahuge" => "extrahuge_1m",
        _ => "large_10k",
    }
}

fn open_file(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(path).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = Command::new("xdg-open").arg(path).spawn();
    }
}

#[derive(Clone, Copy)]
enum Library {
    AcadSharp,
    Ezdxf,
}

fn report_for_json(report: &std::path::Path) -> PathBuf {
    report.with_extension("json")
}

fn report_has_library(report: &GuiReport, library: Library) -> bool {
    report
        .sections
        .iter()
        .flat_map(|section| &section.results)
        .any(|result| match library {
            Library::AcadSharp => result.acadsharp_ms.is_some(),
            Library::Ezdxf => result.ezdxf_ms.is_some(),
        })
}

fn library_badge(ui: &mut egui::Ui, name: &str, measured: bool) {
    let (label, color) = if measured {
        (
            format!("{name}  ready"),
            egui::Color32::from_rgb(45, 140, 75),
        )
    } else {
        (format!("{name}  pending"), egui::Color32::from_gray(110))
    };
    ui.colored_label(color, label);
}

fn show_report_section(ui: &mut egui::Ui, section: &GuiSection) {
    egui::Grid::new("timing-grid")
        .striped(true)
        .num_columns(5)
        .show(ui, |ui| {
            ui.strong("Input");
            ui.strong("dxf-rs (ms)");
            ui.strong("acadrust (ms)");
            ui.strong("ACadSharp (ms)");
            ui.strong("ezdxf (ms)");
            ui.end_row();
            for result in &section.results {
                ui.label(&result.label);
                ui.label(format_ms(result.dxf_ms));
                ui.label(format_ms(result.acadrust_ms));
                ui.label(format_ms(result.acadsharp_ms));
                ui.label(format_ms(result.ezdxf_ms));
                ui.end_row();
            }
        });
}

fn format_ms(value: Option<f64>) -> String {
    value
        .filter(|value| value.is_finite())
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "n/a".into())
}
