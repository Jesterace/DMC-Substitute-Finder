use eframe::egui;
use egui::{Color32, RichText, Sense, Stroke, Vec2};
use std::collections::{HashMap, HashSet};
use std::fs;

const APP_TITLE: &str = "FlossFinder v0.3 - DMC Substitute Finder";
const DMC_CSV: &str = include_str!("dmc_colors.csv");
const STASH_FILE: &str = "flossfinder_stash.txt";

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

#[derive(Clone, Debug)]
struct StashParseResult {
    indexes: HashSet<usize>,
    quantities: HashMap<usize, u32>,
    recognized_codes: Vec<String>,
    unknown_codes: Vec<String>,
    total_skeins: u32,
}

struct SubstituteApp {
    colors: Vec<DmcColor>,
    input_code: String,
    match_count: usize,
    target_index: Option<usize>,
    matches: Vec<MatchResult>,
    selected_match: Option<usize>,
    status: String,
    stash_only: bool,
    stash_text: String,
    stash_status: String,
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
            stash_only: false,
            stash_text: String::new(),
            stash_status: "Optional: paste the DMC colours you own. Quantities are supported.".to_string(),
        }
    }
}

impl eframe::App for SubstituteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(APP_TITLE);
            ui.label("Type the DMC colour you are missing. FlossFinder ranks the closest replacement colours and shows swatches.");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Missing DMC colour:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input_code)
                        .desired_width(150.0)
                        .hint_text("310"),
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
            self.draw_stash_panel(ui);
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
    fn draw_stash_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.stash_only, "My Stash only");
                if ui.button("Load stash").clicked() {
                    self.load_stash();
                }
                if ui.button("Save stash").clicked() {
                    self.save_stash();
                }
                if ui.button("Clear stash").clicked() {
                    self.stash_text.clear();
                    self.stash_status = "Stash cleared.".to_string();
                }
            });

            ui.label("My stash colours. Plain codes mean quantity 1. Add quantities with x, =, or :.");
            ui.add(
                egui::TextEdit::multiline(&mut self.stash_text)
                    .desired_rows(5)
                    .hint_text("310 x2\n666=1\n823:3\nB5200 x1\n3812, 3810, 3847"),
            );

            let parsed = parse_stash(&self.colors, &self.stash_text);
            let unknown_preview = if parsed.unknown_codes.is_empty() {
                String::new()
            } else {
                let preview = parsed
                    .unknown_codes
                    .iter()
                    .take(6)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(" Unknown: {preview}")
            };

            ui.label(format!(
                "{} recognized stash colours, {} total skeins.{}",
                parsed.recognized_codes.len(),
                parsed.total_skeins,
                unknown_preview
            ));
            ui.label(&self.stash_status);
        });
    }

    fn load_stash(&mut self) {
        match fs::read_to_string(STASH_FILE) {
            Ok(text) => {
                self.stash_text = text;
                self.stash_status = format!("Loaded stash from {STASH_FILE}.");
            }
            Err(err) => {
                self.stash_status = format!("Could not load {STASH_FILE}: {err}");
            }
        }
    }

    fn save_stash(&mut self) {
        match fs::write(STASH_FILE, self.stash_text.trim()) {
            Ok(_) => {
                self.stash_status = format!("Saved stash to {STASH_FILE}.");
            }
            Err(err) => {
                self.stash_status = format!("Could not save {STASH_FILE}: {err}");
            }
        }
    }

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

        let stash = parse_stash(&self.colors, &self.stash_text);
        if self.stash_only && stash.indexes.is_empty() {
            self.matches.clear();
            self.selected_match = None;
            self.status = "My Stash only is enabled, but your stash is empty or has no recognized DMC colours.".to_string();
            return;
        }

        let allowed = if self.stash_only {
            Some(&stash.indexes)
        } else {
            None
        };

        self.matches = best_matches(&self.colors, target_index, self.match_count.clamp(3, 25), allowed);
        self.selected_match = if self.matches.is_empty() { None } else { Some(0) };

        let target = &self.colors[target_index];
        if self.stash_only {
            if self.matches.is_empty() {
                self.status = format!(
                    "No stash substitutes found for DMC {}. Add more colours to your stash or turn off My Stash only.",
                    target.code
                );
            } else {
                self.status = format!(
                    "Showing closest substitutes for DMC {} — {} — {} using your {} stash colours / {} total skeins.",
                    target.code,
                    target.name,
                    target.hex,
                    stash.recognized_codes.len(),
                    stash.total_skeins
                );
            }
        } else {
            self.status = format!(
                "Showing closest substitutes for DMC {} — {} — {} using the full DMC list.",
                target.code, target.name, target.hex
            );
        }
    }

    fn draw_results(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            egui::Grid::new("matches_grid")
                .striped(true)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.strong("#");
                    ui.strong("Colour");
                    ui.strong("DMC");
                    ui.strong("Owned");
                    ui.strong("Name");
                    ui.strong("Hex");
                    ui.strong("Closeness");
                    ui.strong("Action");
                    ui.end_row();

                    let stash = parse_stash(&self.colors, &self.stash_text);

                    for row_index in 0..self.matches.len() {
                        let result = &self.matches[row_index];
                        let color = &self.colors[result.color_index];
                        let selected = self.selected_match == Some(row_index);

                        let code = color.code.clone();
                        let name = color.name.clone();
                        let hex = color.hex.clone();
                        let swatch = color.color32();
                        let distance = result.distance;
                        let owned = stash
                            .quantities
                            .get(&result.color_index)
                            .map(|quantity| format!("x{quantity}"))
                            .unwrap_or_else(|| "-".to_string());

                        ui.label((row_index + 1).to_string());
                        draw_swatch(ui, swatch, 58.0, 24.0);

                        let code_text = if selected {
                            RichText::new(&code).strong()
                        } else {
                            RichText::new(&code)
                        };
                        ui.label(code_text);
                        ui.label(owned);
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

fn best_matches(
    colors: &[DmcColor],
    target_index: usize,
    count: usize,
    allowed_indexes: Option<&HashSet<usize>>,
) -> Vec<MatchResult> {
    let target = &colors[target_index];
    let target_code = normalize_code(&target.code);

    let mut matches: Vec<MatchResult> = colors
        .iter()
        .enumerate()
        .filter(|(index, color)| {
            normalize_code(&color.code) != target_code
                && allowed_indexes.map_or(true, |allowed| allowed.contains(index))
        })
        .map(|(index, color)| MatchResult {
            color_index: index,
            distance: lab_distance(target.lab, color.lab),
        })
        .collect();

    matches.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
    matches.truncate(count);
    matches
}

fn parse_stash(colors: &[DmcColor], input: &str) -> StashParseResult {
    let mut quantities: HashMap<usize, u32> = HashMap::new();
    let mut unknown_codes = Vec::new();

    for line in input.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        for entry in split_stash_line(line) {
            let cleaned = entry.trim();
            if cleaned.is_empty() {
                continue;
            }

            let (code, quantity) = parse_stash_entry(cleaned);
            let normalized = normalize_code(&code);
            if normalized.is_empty() {
                continue;
            }

            if let Some(index) = find_color_index(colors, &code) {
                let current = quantities.entry(index).or_insert(0);
                *current = current.saturating_add(quantity.max(1));
            } else {
                unknown_codes.push(cleaned.to_string());
            }
        }
    }

    let indexes: HashSet<usize> = quantities.keys().copied().collect();
    let total_skeins = quantities.values().copied().sum();
    let mut recognized_codes: Vec<String> = quantities
        .iter()
        .map(|(index, quantity)| format!("{} x{}", colors[*index].code, quantity))
        .collect();

    recognized_codes.sort_by(|a, b| {
        let a_code = a.split_whitespace().next().unwrap_or(a);
        let b_code = b.split_whitespace().next().unwrap_or(b);
        naturalish_code_sort(a_code).cmp(&naturalish_code_sort(b_code))
    });
    unknown_codes.sort();

    StashParseResult {
        indexes,
        quantities,
        recognized_codes,
        unknown_codes,
        total_skeins,
    }
}

fn split_stash_line(line: &str) -> Vec<String> {
    let mut entries = Vec::new();

    for group in line.split(|ch| ch == ';' || ch == '|') {
        let group = group.trim();
        if group.is_empty() {
            continue;
        }

        let comma_parts: Vec<&str> = group
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect();

        if comma_parts.len() == 2 && parse_quantity(comma_parts[1]).is_some() {
            entries.push(group.to_string());
        } else if comma_parts.len() > 1 {
            entries.extend(comma_parts.into_iter().map(|part| part.to_string()));
        } else {
            entries.push(group.to_string());
        }
    }

    entries
}

fn parse_stash_entry(input: &str) -> (String, u32) {
    let cleaned = input.trim();
    if cleaned.is_empty() {
        return (String::new(), 1);
    }

    for separator in ['=', ':', ','] {
        if let Some((left, right)) = cleaned.rsplit_once(separator) {
            if let Some(quantity) = parse_quantity(right) {
                return (extract_code(left), quantity.max(1));
            }
        }
    }

    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    if tokens.len() >= 2 {
        if let Some(last) = tokens.last() {
            if let Some(quantity) = parse_quantity(last) {
                let code_part = tokens[..tokens.len() - 1].join(" ");
                return (extract_code(&code_part), quantity.max(1));
            }
        }
    }

    if let Some((code_part, quantity)) = split_trailing_x_quantity(cleaned) {
        return (extract_code(&code_part), quantity.max(1));
    }

    (extract_code(cleaned), 1)
}

fn parse_quantity(input: &str) -> Option<u32> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let trimmed = trimmed
        .trim_start_matches('x')
        .trim_start_matches('X')
        .trim_start_matches("qty")
        .trim_start_matches("Qty")
        .trim_start_matches("QTY")
        .trim_start_matches('=')
        .trim_start_matches(':')
        .trim();

    if trimmed.is_empty() || !trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    trimmed.parse::<u32>().ok()
}

fn split_trailing_x_quantity(input: &str) -> Option<(String, u32)> {
    let lower = input.to_lowercase();
    let x_position = lower.rfind('x')?;
    let (code_part, quantity_part_with_x) = input.split_at(x_position);
    let quantity_part = quantity_part_with_x.get(1..)?.trim();

    if code_part.trim().is_empty() {
        return None;
    }

    parse_quantity(quantity_part).map(|quantity| (code_part.trim().to_string(), quantity))
}

fn naturalish_code_sort(code: &str) -> (u8, u32, String) {
    let normalized = normalize_code(code);
    if let Ok(number) = normalized.parse::<u32>() {
        (0, number, normalized)
    } else {
        (1, 0, normalized)
    }
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
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        native_options,
        Box::new(|_cc| Box::<SubstituteApp>::default()),
    )
}
