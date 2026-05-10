use eframe::egui;
use egui::{Color32, RichText, Sense, Stroke, Vec2};

const APP_TITLE: &str = "DMC Substitute Finder Rust v0.2";
const DMC_CSV: &str = include_str!("dmc_colors.csv");

#[derive(Clone, Debug)]
struct DmcColor {
    code: String,
    name: String,
    hex: String,
    rgb: (u8, u8, u8),
    lab: (f64, f64, f64),
}

#[derive(Clone, Debug)]
struct MatchResult {
    color_index: usize,
    distance: f64,
}

struct SubstituteApp {
    colors: Vec<DmcColor>,
    input_code: String,
    match_count: usize,
    target_index: Option<usize>,
    matches: Vec<MatchResult>,
    selected_match: Option<usize>,
    status: String,
}

impl Default for SubstituteApp {
    fn default() -> Self {
        let colors = load_colors();
        Self {
            colors,
            input_code: String::new(),
            match_count: 10,
            target_index: None,
            matches: Vec::new(),
            selected_match: None,
            status: "Enter a DMC number like 310, 823, B5200, Blanc, or Ecru.".to_string(),
        }
    }
}

impl eframe::App for SubstituteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(APP_TITLE);
            ui.label("Type the DMC colour you are missing. The app ranks the closest replacement colours and shows swatches.");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Missing DMC colour:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input_code)
                        .desired_width(150.0)
                        .hint_text("310")
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.search();
                }

                ui.label("Matches:");
                ui.add(egui::DragValue::new(&mut self.match_count).clamp_range(3..=25));

                if ui.button("Find Substitutes").clicked() {
                    self.search();
                }
            });

            ui.add_space(8.0);
            ui.label(&self.status);
            ui.separator();

            ui.heading("Original colour");
            if let Some(index) = self.target_index {
                let color = &self.colors[index];
                color_info_row(ui, color, 90.0, 34.0);
            } else {
                ui.label("No colour selected yet.");
            }

            ui.add_space(8.0);
            ui.separator();
            ui.heading("Closest substitutions");
            ui.label("Lower closeness number = closer visual match.");
            ui.add_space(4.0);

            if self.matches.is_empty() {
                ui.label("No substitutions shown yet.");
            } else {
                self.draw_results(ui);
            }
        });
    }
}

impl SubstituteApp {
    fn search(&mut self) {
        let entered = extract_code(&self.input_code);
        if entered.is_empty() {
            self.status = "Type a DMC colour first, like 310, 823, B5200, Blanc, White, or Ecru.".to_string();
            return;
        }

        let Some(target_index) = find_color_index(&self.colors, &entered) else {
            self.target_index = None;
            self.matches.clear();
            self.selected_match = None;
            self.status = format!("I could not find DMC colour: {entered}");
            return;
        };

        self.target_index = Some(target_index);
        self.matches = best_matches(&self.colors, target_index, self.match_count.clamp(3, 25));
        self.selected_match = if self.matches.is_empty() { None } else { Some(0) };

        let target = &self.colors[target_index];
        self.status = format!(
            "Showing closest substitutes for DMC {} — {} — {}",
            target.code, target.name, target.hex
        );
    }

    fn draw_results(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
            egui::Grid::new("matches_grid")
                .striped(true)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.strong("#");
                    ui.strong("Colour");
                    ui.strong("DMC");
                    ui.strong("Name");
                    ui.strong("Hex");
                    ui.strong("Closeness");
                    ui.strong("Action");
                    ui.end_row();

                    for row_index in 0..self.matches.len() {
                        let result = &self.matches[row_index];
                        let color = &self.colors[result.color_index];
                        let selected = self.selected_match == Some(row_index);

                        let code = color.code.clone();
                        let name = color.name.clone();
                        let hex = color.hex.clone();
                        let swatch = color.color32();
                        let distance = result.distance;

                        ui.label((row_index + 1).to_string());
                        draw_swatch(ui, swatch, 58.0, 24.0);

                        let code_text = if selected {
                            RichText::new(&code).strong()
                        } else {
                            RichText::new(&code)
                        };
                        ui.label(code_text);
                        ui.label(name);
                        ui.label(hex);
                        ui.label(format!("{distance:.1}"));

                        ui.horizontal(|ui| {
                            if ui.selectable_label(selected, "Select").clicked() {
                                self.selected_match = Some(row_index);
                            }
                            if ui.button("Copy").clicked() {
                                ui.output_mut(|output| output.copied_text = code.clone());
                                self.status = format!("Copied DMC {code} to the clipboard.");
                                self.selected_match = Some(row_index);
                            }
                        });
                        ui.end_row();
                    }
                });
        });

        ui.add_space(10.0);
        ui.separator();
        ui.heading("Selected substitution colour");
        if let Some(selected) = self.selected_match {
            if let Some(result) = self.matches.get(selected) {
                let color = &self.colors[result.color_index];
                color_info_row(ui, color, 90.0, 34.0);
            }
        } else {
            ui.label("No substitute selected yet.");
        }
    }
}

impl DmcColor {
    fn color32(&self) -> Color32 {
        Color32::from_rgb(self.rgb.0, self.rgb.1, self.rgb.2)
    }
}

fn color_info_row(ui: &mut egui::Ui, color: &DmcColor, width: f32, height: f32) {
    ui.horizontal(|ui| {
        draw_swatch(ui, color.color32(), width, height);
        ui.label(format!("DMC {} — {} — {}", color.code, color.name, color.hex));
    });
}

fn draw_swatch(ui: &mut egui::Ui, color: Color32, width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    ui.painter().rect_filled(rect, 3.0, color);
    ui.painter().rect_stroke(rect, 3.0, Stroke::new(1.0, Color32::DARK_GRAY));
}

fn load_colors() -> Vec<DmcColor> {
    let mut colors = Vec::new();

    for (line_number, line) in DMC_CSV.lines().enumerate() {
        if line_number == 0 || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            continue;
        }

        let code = parts[0].trim().to_string();
        let name = parts[1].trim().to_string();
        let hex = parts[2].trim().to_string();

        if let Some(rgb) = hex_to_rgb(&hex) {
            let lab = rgb_to_lab(rgb);
            colors.push(DmcColor { code, name, hex, rgb, lab });
        }
    }

    colors
}

fn find_color_index(colors: &[DmcColor], code: &str) -> Option<usize> {
    let wanted = normalize_code(code);
    colors.iter().position(|color| normalize_code(&color.code) == wanted)
}

fn best_matches(colors: &[DmcColor], target_index: usize, count: usize) -> Vec<MatchResult> {
    let target = &colors[target_index];
    let target_code = normalize_code(&target.code);

    let mut matches: Vec<MatchResult> = colors
        .iter()
        .enumerate()
        .filter(|(_, color)| normalize_code(&color.code) != target_code)
        .map(|(index, color)| MatchResult {
            color_index: index,
            distance: lab_distance(target.lab, color.lab),
        })
        .collect();

    matches.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
    matches.truncate(count);
    matches
}

fn extract_code(input: &str) -> String {
    let mut value = input.trim().to_string();
    if let Some((first, _)) = value.split_once(" - ") {
        value = first.trim().to_string();
    }

    let normalized = normalize_code(&value);
    match normalized.as_str() {
        "WHITE" => "Blanc".to_string(),
        "OFFWHITE" | "OFF-WHITE" | "CREAM" => "Ecru".to_string(),
        _ => value,
    }
}

fn normalize_code(value: &str) -> String {
    value.trim().to_uppercase().replace(' ', "")
}

fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some((r, g, b))
}

fn pivot_rgb(value: u8) -> f64 {
    let value = value as f64 / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn rgb_to_xyz(rgb: (u8, u8, u8)) -> (f64, f64, f64) {
    let r = pivot_rgb(rgb.0);
    let g = pivot_rgb(rgb.1);
    let b = pivot_rgb(rgb.2);

    let x = r * 0.4124564 + g * 0.3575761 + b * 0.1804375;
    let y = r * 0.2126729 + g * 0.7151522 + b * 0.0721750;
    let z = r * 0.0193339 + g * 0.1191920 + b * 0.9503041;

    (x * 100.0, y * 100.0, z * 100.0)
}

fn pivot_xyz(value: f64) -> f64 {
    if value > 0.008856 {
        value.powf(1.0 / 3.0)
    } else {
        (7.787 * value) + (16.0 / 116.0)
    }
}

fn rgb_to_lab(rgb: (u8, u8, u8)) -> (f64, f64, f64) {
    let (mut x, mut y, mut z) = rgb_to_xyz(rgb);

    x /= 95.047;
    y /= 100.000;
    z /= 108.883;

    let fx = pivot_xyz(x);
    let fy = pivot_xyz(y);
    let fz = pivot_xyz(z);

    let l = (116.0 * fy) - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);

    (l, a, b)
}

fn lab_distance(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    let dl = a.0 - b.0;
    let da = a.1 - b.1;
    let db = a.2 - b.2;
    (dl * dl + da * da + db * db).sqrt()
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        native_options,
        Box::new(|_cc| Box::<SubstituteApp>::default()),
    )
}
