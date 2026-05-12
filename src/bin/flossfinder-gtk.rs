use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, CssProvider, DrawingArea,
    Entry, FileChooserAction, FileChooserNative, Frame, Label, ListBox, ListBoxRow, Orientation,
    ResponseType, ScrolledWindow, SpinButton, TextView,
};
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

const APP_ID: &str = "com.jesterace.flossfinder.native";
const APP_TITLE: &str = "Floss Finder";
const DMC_CSV: &str = include_str!("../dmc_colors.csv");
const SETTINGS_RELATIVE_PATH: &str = ".config/flossfinder-gtk/settings.txt";

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

struct AppState {
    colors: Vec<DmcColor>,
}

impl AppState {
    fn new() -> Self {
        Self {
            colors: load_colors(),
        }
    }
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(AppState::new()));

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .default_width(980)
        .default_height(720)
        .build();

    let css = CssProvider::new();
    css.load_from_data(
        ".light-root { \
            background: #ffffff; \
            color: #000000; \
        }\n\
        .dark-root { \
            background: #1e1e1e; \
            color: #f2f2f2; \
        }\n\
        .dark-root * { \
            color: #f2f2f2; \
        }\n\
        .dark-root entry, \
        .dark-root spinbutton, \
        .dark-root listbox, \
        .dark-root row { \
            background: #2a2a2a; \
            color: #f2f2f2; \
        }\n\
        .dark-root button { \
            background: #3a3a3a; \
            color: #ffffff; \
        }\n\
        .dark-root .results-scroll, \
        .dark-root .results-scroll viewport, \
        .dark-root .results-list, \
        .dark-root .results-list row { \
            background: #1e1e1e; \
            color: #f2f2f2; \
        }\n\
        .dark-root .results-list label { \
            color: #f2f2f2; \
        }\n\
        .stash-box { \
            border: 2px solid #d0d0d0; \
            border-radius: 8px; \
            background: #ffffff; \
            padding: 6px; \
        }\n\
        .stash-box viewport { \
            background: #ffffff; \
        }\n\
        textview.stash-text { \
            background: #ffffff; \
            color: #000000; \
            padding: 8px; \
            font-size: 14px; \
        }\n\
        textview.stash-text text { \
            background: #ffffff; \
            color: #000000; \
        }\n\
        textview.stash-text > text { \
            background: #ffffff; \
            color: #000000; \
        }\n\
        .dark-root .stash-box { \
            border: 2px solid #777777; \
            background: #d8d8d8; \
        }\n\
        .dark-root .stash-box viewport { \
            background: #d8d8d8; \
        }\n\
        .dark-root textview.stash-text { \
            background: #d8d8d8; \
            color: #000000; \
        }\n\
        .dark-root textview.stash-text text { \
            background: #d8d8d8; \
            color: #000000; \
        }\n\
        .dark-root textview.stash-text > text { \
            background: #d8d8d8; \
            color: #000000; \
        }"
    );

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let main_box = GtkBox::new(Orientation::Vertical, 10);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);
    set_root_dark_class(&main_box, load_dark_mode_setting());

    let title_row = GtkBox::new(Orientation::Horizontal, 8);

    let title = Label::new(Some(APP_TITLE));
    title.set_xalign(0.0);
    title.set_hexpand(true);
    title.add_css_class("title-1");

    let settings_button = Button::with_label("⚙ Settings");

    title_row.append(&title);
    title_row.append(&settings_button);
    main_box.append(&title_row);

    let subtitle = Label::new(Some(
        "Type the DMC colour you are missing. Floss Finder ranks the closest replacement colours.",
    ));
    subtitle.set_xalign(0.0);
    main_box.append(&subtitle);

    let settings_panel = Frame::new(Some("Settings"));
    settings_panel.set_visible(false);

    let settings_box = GtkBox::new(Orientation::Vertical, 8);
    settings_box.set_margin_top(10);
    settings_box.set_margin_bottom(10);
    settings_box.set_margin_start(10);
    settings_box.set_margin_end(10);

    let dark_mode_toggle = CheckButton::with_label("Dark mode");
    dark_mode_toggle.set_active(load_dark_mode_setting());
    apply_dark_mode(dark_mode_toggle.is_active());

    let theme_note = Label::new(Some("Theme setting is saved automatically."));
    theme_note.set_xalign(0.0);

    settings_box.append(&dark_mode_toggle);
    settings_box.append(&theme_note);

    settings_panel.set_child(Some(&settings_box));
    main_box.append(&settings_panel);

    let search_row = GtkBox::new(Orientation::Horizontal, 8);

    let input_label = Label::new(Some("Missing DMC colour:"));
    let input_entry = Entry::new();
    input_entry.set_placeholder_text(Some("310, 823, B5200, Blanc, Ecru"));

    let match_label = Label::new(Some("Matches:"));
    let match_spin = SpinButton::with_range(3.0, 25.0, 1.0);
    match_spin.set_value(10.0);

    let find_button = Button::with_label("Find Substitutes");

    search_row.append(&input_label);
    search_row.append(&input_entry);
    search_row.append(&match_label);
    search_row.append(&match_spin);
    search_row.append(&find_button);

    main_box.append(&search_row);

    let stash_frame = GtkBox::new(Orientation::Vertical, 6);

    let stash_controls = GtkBox::new(Orientation::Horizontal, 8);

    let stash_toggle = CheckButton::with_label("My Stash only");
    let load_flosskeeper_button = Button::with_label("Load FlossKeeper stash");
    let choose_stash_button = Button::with_label("Choose stash file");
    let clear_stash_button = Button::with_label("Clear stash");

    stash_controls.append(&stash_toggle);
    stash_controls.append(&load_flosskeeper_button);
    stash_controls.append(&choose_stash_button);
    stash_controls.append(&clear_stash_button);

    stash_frame.append(&stash_controls);

    let stash_help = Label::new(Some(
        "Optional stash list. Examples: 310 x2, 666=1, 823:3, B5200 x1, 3812, 3810",
    ));
    stash_help.set_xalign(0.0);
    stash_frame.append(&stash_help);

    let stash_view = TextView::new();
    stash_view.set_monospace(true);
    stash_view.set_wrap_mode(gtk::WrapMode::WordChar);
    stash_view.set_vexpand(true);
    stash_view.add_css_class("stash-text");
    stash_view.buffer().set_text("");

    let stash_scroll = ScrolledWindow::new();
    stash_scroll.set_min_content_height(170);
    stash_scroll.set_vexpand(false);
    stash_scroll.add_css_class("stash-box");
    stash_scroll.set_child(Some(&stash_view));

    let stash_text_frame = Frame::new(Some("Stash list"));
    stash_text_frame.set_child(Some(&stash_scroll));
    stash_frame.append(&stash_text_frame);

    main_box.append(&stash_frame);

    let status_label = Label::new(Some("Enter a DMC number to begin."));
    status_label.set_xalign(0.0);
    main_box.append(&status_label);

    let original_label = Label::new(Some("Original colour: none selected"));
    original_label.set_xalign(0.0);
    original_label.add_css_class("heading");
    main_box.append(&original_label);

    let results_heading = Label::new(Some("Closest substitutions"));
    results_heading.set_xalign(0.0);
    results_heading.add_css_class("heading");
    main_box.append(&results_heading);

    let results_list = ListBox::new();
    results_list.add_css_class("results-list");

    let results_scroll = ScrolledWindow::new();
    results_scroll.add_css_class("results-scroll");
    results_scroll.set_vexpand(true);
    results_scroll.set_child(Some(&results_list));
    main_box.append(&results_scroll);

    window.set_child(Some(&main_box));

    {
        let parent_window = window.clone();
        let status_label = status_label.clone();
        let main_box = main_box.clone();

        settings_button.connect_clicked(move |_| {
            let settings_window = gtk::Window::builder()
                .title("Floss Finder Settings")
                .default_width(360)
                .default_height(180)
                .transient_for(&parent_window)
                .modal(true)
                .build();

            let settings_box = GtkBox::new(Orientation::Vertical, 10);
            settings_box.set_margin_top(14);
            settings_box.set_margin_bottom(14);
            settings_box.set_margin_start(14);
            settings_box.set_margin_end(14);
            set_root_dark_class(&settings_box, load_dark_mode_setting());

            let title = Label::new(Some("Settings"));
            title.set_xalign(0.0);
            title.add_css_class("title-1");
            settings_box.append(&title);

            let dark_mode_toggle = CheckButton::with_label("Dark mode");
            dark_mode_toggle.set_active(load_dark_mode_setting());
            settings_box.append(&dark_mode_toggle);

            let note = Label::new(Some("Theme setting is saved automatically."));
            note.set_xalign(0.0);
            settings_box.append(&note);

            let close_button = Button::with_label("Close");
            settings_box.append(&close_button);

            {
                let status_label = status_label.clone();
                let main_box = main_box.clone();
                let settings_box = settings_box.clone();

                dark_mode_toggle.connect_toggled(move |button| {
                    let enabled = button.is_active();
                    apply_dark_mode(enabled);
                    set_root_dark_class(&main_box, enabled);
                    set_root_dark_class(&settings_box, enabled);
                    save_dark_mode_setting(enabled);

                    if enabled {
                        status_label.set_text("Dark mode enabled.");
                    } else {
                        status_label.set_text("Light mode enabled.");
                    }
                });
            }

            {
                let settings_window = settings_window.clone();

                close_button.connect_clicked(move |_| {
                    settings_window.close();
                });
            }

            settings_window.set_child(Some(&settings_box));
            settings_window.present();
        });
    }

    {
        let status_label = status_label.clone();

        dark_mode_toggle.connect_toggled(move |button| {
            let enabled = button.is_active();
            apply_dark_mode(enabled);
            save_dark_mode_setting(enabled);

            if enabled {
                status_label.set_text("Dark mode enabled.");
            } else {
                status_label.set_text("Light mode enabled.");
            }
        });
    }

    {
        let state = state.clone();
        let input_entry = input_entry.clone();
        let match_spin = match_spin.clone();
        let stash_toggle = stash_toggle.clone();
        let stash_view = stash_view.clone();
        let status_label = status_label.clone();
        let original_label = original_label.clone();
        let results_list = results_list.clone();

        find_button.connect_clicked(move |_| {
            run_search(
                &state,
                &input_entry,
                &match_spin,
                &stash_toggle,
                &stash_view,
                &status_label,
                &original_label,
                &results_list,
            );
        });
    }

    {
        let state = state.clone();
        let input_entry_clone = input_entry.clone();
        let match_spin = match_spin.clone();
        let stash_toggle = stash_toggle.clone();
        let stash_view = stash_view.clone();
        let status_label = status_label.clone();
        let original_label = original_label.clone();
        let results_list = results_list.clone();

        input_entry.connect_activate(move |_| {
            run_search(
                &state,
                &input_entry_clone,
                &match_spin,
                &stash_toggle,
                &stash_view,
                &status_label,
                &original_label,
                &results_list,
            );
        });
    }

    {
        let stash_view = stash_view.clone();
        let status_label = status_label.clone();

        load_flosskeeper_button.connect_clicked(move |_| {
            match load_flosskeeper_stash_text() {
                Ok(text) => {
                    stash_view.buffer().set_text(&text);
                    status_label.set_text("Loaded stash from FlossKeeper.");
                }
                Err(err) => {
                    status_label.set_text(&err);
                }
            }
        });
    }

    {
        let stash_view = stash_view.clone();
        let status_label = status_label.clone();
        let parent_window = window.clone();

        choose_stash_button.connect_clicked(move |_| {
            let dialog = FileChooserNative::new(
                Some("Choose FlossKeeper stash file"),
                Some(&parent_window),
                FileChooserAction::Open,
                Some("Open"),
                Some("Cancel"),
            );

            let stash_view = stash_view.clone();
            let status_label = status_label.clone();

            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            match load_flosskeeper_stash_text_from_path(&path) {
                                Ok(text) => {
                                    stash_view.buffer().set_text(&text);
                                    status_label.set_text(&format!(
                                        "Loaded stash from {}.",
                                        path.display()
                                    ));
                                }
                                Err(err) => {
                                    status_label.set_text(&err);
                                }
                            }
                        } else {
                            status_label.set_text("Could not get a local file path from the selected file.");
                        }
                    }
                }

                dialog.destroy();
            });

            dialog.show();
        });
    }

    {
        let stash_view = stash_view.clone();
        let status_label = status_label.clone();

        clear_stash_button.connect_clicked(move |_| {
            stash_view.buffer().set_text("");
            status_label.set_text("Stash list cleared.");
        });
    }

    window.present();
}


fn set_root_dark_class(root: &GtkBox, enabled: bool) {
    root.remove_css_class("light-root");
    root.remove_css_class("dark-root");

    if enabled {
        root.add_css_class("dark-root");
    } else {
        root.add_css_class("light-root");
    }
}

fn settings_file_path() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .map(|home| format!("{home}/{SETTINGS_RELATIVE_PATH}"))
}

fn load_dark_mode_setting() -> bool {
    let Some(path) = settings_file_path() else {
        return false;
    };

    match fs::read_to_string(path) {
        Ok(text) => text.trim().eq_ignore_ascii_case("dark=true"),
        Err(_) => false,
    }
}

fn save_dark_mode_setting(enabled: bool) {
    let Some(path) = settings_file_path() else {
        return;
    };

    if let Some(parent) = Path::new(&path).parent() {
        let _ = fs::create_dir_all(parent);
    }

    let value = if enabled { "dark=true\n" } else { "dark=false\n" };
    let _ = fs::write(path, value);
}

fn apply_dark_mode(enabled: bool) {
    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(enabled);
    }
}

fn load_flosskeeper_stash_text() -> Result<String, String> {
    let home = std::env::var("HOME")
        .map_err(|_| "Could not find your HOME folder.".to_string())?;

    let path = format!("{home}/FlossKeeperSync/flosskeeper_collection.tsv");
    load_flosskeeper_stash_text_from_path(Path::new(&path))
}

fn load_flosskeeper_stash_text_from_path(path: &Path) -> Result<String, String> {
    let input = fs::read_to_string(path).map_err(|err| {
        format!(
            "Could not read FlossKeeper stash file: {}\n{}",
            path.display(),
            err
        )
    })?;

    let mut output = Vec::new();

    for line in input.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 3 {
            continue;
        }

        let code = parts[0].trim();
        let bobbins = parts[1].trim().parse::<u32>().unwrap_or(0);
        let skeins = parts[2].trim().parse::<u32>().unwrap_or(0);
        let total = bobbins.saturating_add(skeins);

        if total > 0 {
            output.push(format!("{code} x{total}"));
        }
    }

    if output.is_empty() {
        return Err("No owned colours found in the selected FlossKeeper stash file.".to_string());
    }

    Ok(format!("{}\n", output.join("\n")))
}


fn run_search(
    state: &Rc<RefCell<AppState>>,
    input_entry: &Entry,
    match_spin: &SpinButton,
    stash_toggle: &CheckButton,
    stash_view: &TextView,
    status_label: &Label,
    original_label: &Label,
    results_list: &ListBox,
) {
    clear_listbox(results_list);

    let entered = extract_code(input_entry.text().as_str());

    if entered.is_empty() {
        status_label.set_text("Type a DMC colour first.");
        original_label.set_text("Original colour: none selected");
        return;
    }

    let stash_text = textview_text(stash_view);
    let match_count = match_spin.value_as_int().clamp(3, 25) as usize;

    let state_ref = state.borrow();
    let colors = &state_ref.colors;

    let Some(target_index) = find_color_index(colors, &entered) else {
        status_label.set_text(&format!("I could not find DMC colour: {entered}"));
        original_label.set_text("Original colour: none selected");
        return;
    };

    let stash = parse_stash(colors, &stash_text);

    let unknown_note = if stash.unknown_codes.is_empty() {
        String::new()
    } else {
        format!(" {} unknown stash entries ignored.", stash.unknown_codes.len())
    };

    if stash_toggle.is_active() && stash.indexes.is_empty() {
        status_label.set_text("My Stash only is enabled, but no stash colours were recognized.");
        original_label.set_text("Original colour: none selected");
        return;
    }

    let allowed = if stash_toggle.is_active() {
        Some(&stash.indexes)
    } else {
        None
    };

    let matches = best_matches(colors, target_index, match_count, allowed);
    let target = &colors[target_index];

    original_label.set_text(&format!(
        "Original colour: DMC {} — {} — {}",
        target.code, target.name, target.hex
    ));

    if stash_toggle.is_active() {
        status_label.set_text(&format!(
            "Showing closest stash substitutes for DMC {} using {} stash colours / {} total skeins.{}",
            target.code,
            stash.recognized_codes.len(),
            stash.total_skeins,
            unknown_note
        ));
    } else {
        status_label.set_text(&format!(
            "Showing closest substitutes for DMC {} using the full DMC list.{}",
            target.code,
            unknown_note
        ));
    }

    if matches.is_empty() {
        let row = ListBoxRow::new();
        row.set_child(Some(&Label::new(Some("No substitutions found."))));
        results_list.append(&row);
        return;
    }

    for (row_index, result) in matches.iter().enumerate() {
        let color = colors[result.color_index].clone();

        let row = ListBoxRow::new();
        let row_box = GtkBox::new(Orientation::Horizontal, 10);
        row_box.set_margin_top(6);
        row_box.set_margin_bottom(6);
        row_box.set_margin_start(6);
        row_box.set_margin_end(6);

        let number = Label::new(Some(&(row_index + 1).to_string()));
        number.set_width_chars(3);

        let swatch = make_swatch(color.rgb);

        let owned = stash
            .quantities
            .get(&result.color_index)
            .map(|qty| format!("x{qty}"))
            .unwrap_or_else(|| "-".to_string());

        let info = Label::new(Some(&format!(
            "DMC {}   {}   {}   Owned: {}   Closeness: {:.1}",
            color.code, color.name, color.hex, owned, result.distance
        )));
        info.set_xalign(0.0);
        info.set_hexpand(true);

        let copy_button = Button::with_label("Copy");
        let copy_code = color.code.clone();
        copy_button.connect_clicked(move |_| {
            if let Some(display) = gtk::gdk::Display::default() {
                display.clipboard().set_text(&copy_code);
            }
        });

        row_box.append(&number);
        row_box.append(&swatch);
        row_box.append(&info);
        row_box.append(&copy_button);

        row.set_child(Some(&row_box));
        results_list.append(&row);
    }
}

fn clear_listbox(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn textview_text(view: &TextView) -> String {
    let buffer = view.buffer();
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    buffer.text(&start, &end, true).to_string()
}

fn make_swatch(rgb: (u8, u8, u8)) -> DrawingArea {
    let area = DrawingArea::new();
    area.set_content_width(58);
    area.set_content_height(26);

    area.set_draw_func(move |_, cr, width, height| {
        cr.set_source_rgb(
            rgb.0 as f64 / 255.0,
            rgb.1 as f64 / 255.0,
            rgb.2 as f64 / 255.0,
        );
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        let _ = cr.fill();

        cr.set_source_rgb(0.25, 0.25, 0.25);
        cr.rectangle(0.5, 0.5, width as f64 - 1.0, height as f64 - 1.0);
        let _ = cr.stroke();
    });

    area
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
            colors.push(DmcColor {
                code,
                name,
                hex,
                rgb,
                lab,
            });
        }
    }

    colors
}

fn find_color_index(colors: &[DmcColor], code: &str) -> Option<usize> {
    let wanted = normalize_code(code);
    colors
        .iter()
        .position(|color| normalize_code(&color.code) == wanted)
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

    matches.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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

    recognized_codes.sort();
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
