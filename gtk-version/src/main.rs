use gtk::gdk;
use gtk::prelude::*;
use gtk::{
    Adjustment, Application, ApplicationWindow, Box as GtkBox, Button, DrawingArea, Entry, Grid,
    Label, ListBox, Orientation, ScrolledWindow, SpinButton,
};
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "com.jaredmackenzie.flossfinder.gtk";
const APP_TITLE: &str = "FlossFinder GTK - DMC Substitute Finder";
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

struct AppState {
    colors: Vec<DmcColor>,
    target_index: Option<usize>,
    matches: Vec<MatchResult>,
    selected_match: Option<usize>,
}

#[derive(Clone)]
struct UiHandles {
    entry: Entry,
    match_count: SpinButton,
    status: Label,
    original_box: GtkBox,
    results_list: ListBox,
    selected_box: GtkBox,
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(AppState {
        colors: load_colors(),
        target_index: None,
        matches: Vec::new(),
        selected_match: None,
    }));

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .default_width(920)
        .default_height(650)
        .build();

    let main = GtkBox::new(Orientation::Vertical, 10);
    main.set_margin_top(14);
    main.set_margin_bottom(14);
    main.set_margin_start(14);
    main.set_margin_end(14);

    let title = Label::new(Some(APP_TITLE));
    title.set_xalign(0.0);
    title.set_markup("<span size='x-large' weight='bold'>FlossFinder GTK</span>");
    main.append(&title);

    let help = Label::new(Some(
        "Enter the DMC colour you are missing. FlossFinder ranks the closest replacement colours and shows swatches.",
    ));
    help.set_xalign(0.0);
    help.set_wrap(true);
    main.append(&help);

    let controls = GtkBox::new(Orientation::Horizontal, 8);

    let entry_label = Label::new(Some("Missing DMC:"));
    controls.append(&entry_label);

    let entry = Entry::builder()
        .placeholder_text("310, 823, B5200, Blanc, Ecru")
        .width_chars(22)
        .build();
    controls.append(&entry);

    let count_label = Label::new(Some("Matches:"));
    controls.append(&count_label);

    let adjustment = Adjustment::new(10.0, 3.0, 25.0, 1.0, 5.0, 0.0);
    let match_count = SpinButton::new(Some(&adjustment), 1.0, 0);
    controls.append(&match_count);

    let find_button = Button::with_label("Find Substitutes");
    controls.append(&find_button);

    main.append(&controls);

    let status = Label::new(Some("Enter a DMC number like 310, 823, B5200, Blanc, or Ecru."));
    status.set_xalign(0.0);
    status.set_wrap(true);
    main.append(&status);

    let original_heading = Label::new(Some("Original colour"));
    original_heading.set_xalign(0.0);
    original_heading.set_markup("<b>Original colour</b>");
    main.append(&original_heading);

    let original_box = GtkBox::new(Orientation::Horizontal, 8);
    original_box.append(&Label::new(Some("No colour selected yet.")));
    main.append(&original_box);

    let sub_heading = Label::new(Some("Closest substitutions"));
    sub_heading.set_xalign(0.0);
    sub_heading.set_markup("<b>Closest substitutions</b>");
    main.append(&sub_heading);

    let note = Label::new(Some("Lower closeness number = closer visual match."));
    note.set_xalign(0.0);
    main.append(&note);

    let results_list = ListBox::new();
    results_list.set_vexpand(true);

    let scrolled = ScrolledWindow::builder()
        .min_content_height(300)
        .vexpand(true)
        .child(&results_list)
        .build();
    main.append(&scrolled);

    let selected_heading = Label::new(Some("Selected substitution colour"));
    selected_heading.set_xalign(0.0);
    selected_heading.set_markup("<b>Selected substitution colour</b>");
    main.append(&selected_heading);

    let selected_box = GtkBox::new(Orientation::Horizontal, 8);
    selected_box.append(&Label::new(Some("No substitute selected yet.")));
    main.append(&selected_box);

    let handles = Rc::new(UiHandles {
        entry,
        match_count,
        status,
        original_box,
        results_list,
        selected_box,
    });

    {
        let state = state.clone();
        let handles = handles.clone();
        find_button.connect_clicked(move |_| {
            run_search(state.clone(), handles.clone());
        });
    }

    {
        let state = state.clone();
        let handles_for_callback = handles.clone();
        handles.entry.connect_activate(move |_| {
            run_search(state.clone(), handles_for_callback.clone());
        });
    }

    window.set_child(Some(&main));
    window.present();
}

fn run_search(state: Rc<RefCell<AppState>>, handles: Rc<UiHandles>) {
    let entered = extract_code(&handles.entry.text());
    if entered.is_empty() {
        handles
            .status
            .set_text("Type a DMC colour first, like 310, 823, B5200, Blanc, White, or Ecru.");
        return;
    }

    let target_index = {
        let borrowed = state.borrow();
        find_color_index(&borrowed.colors, &entered)
    };

    let Some(target_index) = target_index else {
        {
            let mut borrowed = state.borrow_mut();
            borrowed.target_index = None;
            borrowed.matches.clear();
            borrowed.selected_match = None;
        }
        clear_box(&handles.original_box);
        handles
            .original_box
            .append(&Label::new(Some("No colour selected yet.")));
        clear_box(&handles.selected_box);
        handles
            .selected_box
            .append(&Label::new(Some("No substitute selected yet.")));
        clear_listbox(&handles.results_list);
        handles
            .status
            .set_text(&format!("I could not find DMC colour: {entered}"));
        return;
    };

    let match_count = handles.match_count.value_as_int().clamp(3, 25) as usize;

    let (target, status_text) = {
        let mut borrowed = state.borrow_mut();
        borrowed.target_index = Some(target_index);
        borrowed.matches = best_matches(&borrowed.colors, target_index, match_count);
        borrowed.selected_match = if borrowed.matches.is_empty() { None } else { Some(0) };

        let target = borrowed.colors[target_index].clone();
        let status = format!(
            "Showing closest substitutes for DMC {} — {} — {}",
            target.code, target.name, target.hex
        );
        (target, status)
    };

    handles.status.set_text(&status_text);
    render_color_info(&handles.original_box, &target, 100, 36);
    render_results(state.clone(), handles.clone());
    render_selected(state, handles);
}

fn render_results(state: Rc<RefCell<AppState>>, handles: Rc<UiHandles>) {
    clear_listbox(&handles.results_list);

    let header = Grid::builder()
        .column_spacing(14)
        .row_spacing(4)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .build();
    add_bold_grid_label(&header, "#", 0, 0);
    add_bold_grid_label(&header, "Colour", 1, 0);
    add_bold_grid_label(&header, "DMC", 2, 0);
    add_bold_grid_label(&header, "Name", 3, 0);
    add_bold_grid_label(&header, "Hex", 4, 0);
    add_bold_grid_label(&header, "Closeness", 5, 0);
    add_bold_grid_label(&header, "Action", 6, 0);
    handles.results_list.append(&header);

    let snapshot = {
        let borrowed = state.borrow();
        borrowed
            .matches
            .iter()
            .enumerate()
            .map(|(row_index, result)| {
                let color = borrowed.colors[result.color_index].clone();
                (row_index, color, result.distance)
            })
            .collect::<Vec<_>>()
    };

    if snapshot.is_empty() {
        handles
            .results_list
            .append(&Label::new(Some("No substitutions shown yet.")));
        return;
    }

    for (row_index, color, distance) in snapshot {
        let row = Grid::builder()
            .column_spacing(14)
            .row_spacing(4)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(4)
            .margin_end(4)
            .build();

        row.attach(&Label::new(Some(&(row_index + 1).to_string())), 0, 0, 1, 1);
        row.attach(&make_swatch(color.rgb, 64, 24), 1, 0, 1, 1);
        row.attach(&left_label(&color.code), 2, 0, 1, 1);
        row.attach(&left_label(&color.name), 3, 0, 1, 1);
        row.attach(&left_label(&color.hex), 4, 0, 1, 1);
        row.attach(&left_label(&format!("{distance:.1}")), 5, 0, 1, 1);

        let actions = GtkBox::new(Orientation::Horizontal, 6);
        let select_button = Button::with_label("Select");
        let copy_button = Button::with_label("Copy");

        {
            let state = state.clone();
            let handles = handles.clone();
            select_button.connect_clicked(move |_| {
                state.borrow_mut().selected_match = Some(row_index);
                render_selected(state.clone(), handles.clone());
            });
        }

        {
            let state = state.clone();
            let handles = handles.clone();
            let code = color.code.clone();
            copy_button.connect_clicked(move |_| {
                if let Some(display) = gdk::Display::default() {
                    display.clipboard().set_text(&code);
                }
                state.borrow_mut().selected_match = Some(row_index);
                handles.status.set_text(&format!("Copied DMC {code} to the clipboard."));
                render_selected(state.clone(), handles.clone());
            });
        }

        actions.append(&select_button);
        actions.append(&copy_button);
        row.attach(&actions, 6, 0, 1, 1);

        handles.results_list.append(&row);
    }
}

fn render_selected(state: Rc<RefCell<AppState>>, handles: Rc<UiHandles>) {
    let selected = {
        let borrowed = state.borrow();
        borrowed.selected_match.and_then(|index| {
            borrowed
                .matches
                .get(index)
                .map(|result| borrowed.colors[result.color_index].clone())
        })
    };

    if let Some(color) = selected {
        render_color_info(&handles.selected_box, &color, 100, 36);
    } else {
        clear_box(&handles.selected_box);
        handles
            .selected_box
            .append(&Label::new(Some("No substitute selected yet.")));
    }
}

fn add_bold_grid_label(grid: &Grid, text: &str, column: i32, row: i32) {
    let label = Label::new(None);
    label.set_xalign(0.0);
    label.set_markup(&format!("<b>{text}</b>"));
    grid.attach(&label, column, row, 1, 1);
}

fn left_label(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.set_xalign(0.0);
    label
}

fn clear_box(container: &GtkBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn clear_listbox(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn render_color_info(container: &GtkBox, color: &DmcColor, width: i32, height: i32) {
    clear_box(container);
    container.append(&make_swatch(color.rgb, width, height));
    let label = Label::new(Some(&format!(
        "DMC {} — {} — {}",
        color.code, color.name, color.hex
    )));
    label.set_xalign(0.0);
    container.append(&label);
}

fn make_swatch(rgb: (u8, u8, u8), width: i32, height: i32) -> DrawingArea {
    let swatch = DrawingArea::new();
    swatch.set_content_width(width);
    swatch.set_content_height(height);
    swatch.set_draw_func(move |_area, cr, w, h| {
        cr.set_source_rgb(
            rgb.0 as f64 / 255.0,
            rgb.1 as f64 / 255.0,
            rgb.2 as f64 / 255.0,
        );
        cr.rectangle(0.0, 0.0, w as f64, h as f64);
        let _ = cr.fill();

        cr.set_source_rgb(0.20, 0.20, 0.20);
        cr.rectangle(0.5, 0.5, (w - 1) as f64, (h - 1) as f64);
        let _ = cr.stroke();
    });
    swatch
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
