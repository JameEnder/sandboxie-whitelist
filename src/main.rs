// Hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use egui_file_dialog::FileDialog;
use serde::{Deserialize, Serialize};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 600.0]),

        ..Default::default()
    };

    eframe::run_native(
        "Sandboxie Whitelist Tool",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[derive(Clone, PartialEq, Eq, Serialize, PartialOrd, Ord, Deserialize)]
enum Access {
    Normal,
    Open,
    OpenForAll,
    Closed,
    ReadOnly,
    BoxOnly,
}

impl Access {
    fn to_ini_string(&self) -> String {
        match self {
            Access::Normal => "NormalFilePath",
            Access::Open => "OpenFilePath",
            Access::OpenForAll => "OpenPipePath",
            Access::Closed => "ClosedFilePath",
            Access::ReadOnly => "ReadFilePath",
            Access::BoxOnly => "WriteFilePath",
        }
        .to_string()
    }
}

impl std::fmt::Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Access::Normal => "Normal",
                Access::Open => "Open",
                Access::OpenForAll => "Open For All",
                Access::Closed => "Closed",
                Access::ReadOnly => "Read Only",
                Access::BoxOnly => "Box Only",
            }
        )
    }
}

const ACCESSES: &[(Access, &str)] = &[
    (
        Access::Normal,
        "Regular Sandboxie Behavior - allow read and also copy on write.",
    ),
    (Access::Open, "Allow write-access outside of sandbox."),
    (Access::OpenForAll, "Allow write-access outside of sandbox, also for applications installed inside the sandbox."),
    (Access::Closed, "Deny access to host location and prevent creation of sandboxed copies."),
    (Access::ReadOnly, "Allow read-only access only."),
    (Access::BoxOnly, "Hide host files, folders or registry keys from sandboxed processes."),
];

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Entry {
    path: String,
    access: Access,
}

#[derive(Serialize, Deserialize)]
struct App {
    entries: Vec<Entry>,
    default_access: Access,
    privacy_mode: bool,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    generated: String,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    file_dialog: Option<egui_file_dialog::FileDialog>,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        cc.egui_ctx.set_zoom_factor(2.0);

        if let Ok(contents) = std::fs::read_to_string("config.json") {
            serde_json::from_str(&contents).unwrap()
        } else {
            Self {
                entries: vec![
                    Entry {
                        path: "C:\\Program Files".into(),
                        access: Access::Open,
                    },
                    Entry {
                        path: "C:\\Program Files (x86)".into(),
                        access: Access::Open,
                    },
                    Entry {
                        path: "C:\\Windows\\System32".into(),
                        access: Access::Open,
                    },
                ],
                privacy_mode: true,
                default_access: Access::Open,
                generated: String::new(),
                file_dialog: None,
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(dialog) = &mut self.file_dialog {
            dialog.update(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;

                    ui.heading("Sandboxie Whitelist Tool");

                    if ui.button("Save").clicked() {
                        std::fs::write("config.json", serde_json::to_string_pretty(&self).unwrap())
                            .unwrap();
                    }
                    ui.hyperlink_to("⭐", "https://github.com/jameender/sandboxie-whitelist");
                });
                egui::ComboBox::from_label("Default Access")
                    .selected_text(self.default_access.to_string())
                    .show_ui(ui, |ui| {
                        for (access, label) in ACCESSES.iter() {
                            ui.selectable_value(
                                &mut self.default_access,
                                access.clone(),
                                access.to_string(),
                            )
                            .on_hover_text(label.to_string());
                        }
                    });
                let mut already_seen: Vec<&str> = vec![];
                let mut to_be_deleted: Vec<usize> = vec![];

                ui.vertical(|ui| {
                    for (index, entry) in &mut self.entries.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut entry.path).desired_width(300.0),
                            );
                            let checked = std::fs::metadata(&entry.path).is_ok();

                            if checked {
                                ui.colored_label(egui::Color32::GREEN, "✔");
                            } else {
                                ui.colored_label(egui::Color32::RED, "❌");
                            }

                            if !already_seen.contains(&entry.path.as_ref())
                                && !entry.path.is_empty()
                            {
                                egui::ComboBox::from_id_salt(&entry.path)
                                    .selected_text(entry.access.to_string())
                                    .show_ui(ui, |ui| {
                                        for (access, label) in ACCESSES.iter() {
                                            ui.selectable_value(
                                                &mut entry.access,
                                                access.clone(),
                                                access.to_string(),
                                            )
                                            .on_hover_text(label.to_string());
                                        }
                                    });
                            }
                            if ui.button("X").clicked() {
                                to_be_deleted.push(index);
                            }

                            already_seen.push(&entry.path);
                        });
                    }
                });
                let mut current_index = 0;

                self.entries.retain(|_| {
                    let keep = !to_be_deleted.contains(&current_index);
                    current_index += 1;
                    keep
                });

                ui.horizontal(|ui| {
                    if ui.button("+").clicked() {
                        self.entries.push(Entry {
                            path: String::new(),
                            access: self.default_access.clone(),
                        });
                    }
                    if ui.button("Browse").clicked() {
                        self.file_dialog = Some(FileDialog::new());
                        if let Some(dialog) = &mut self.file_dialog {
                            dialog.select_multiple();
                        }
                    }
                });

                if let Some(dialog) = &mut self.file_dialog {
                    if let Some(paths) = dialog.selected_multiple() {
                        self.entries.extend(paths.iter().map(|path| {
                            Entry {
                                path: path
                                    .canonicalize()
                                    .unwrap()
                                    .to_string_lossy()
                                    .into_owned()
                                    .replace("\\\\?\\", ""),
                                access: self.default_access.clone(),
                            }
                        }));

                        self.file_dialog = None;
                    }
                }
                ui.checkbox(&mut self.privacy_mode, "Privacy Mode")
                    .on_hover_text("Put every other entry in the blacklist");

                ui.horizontal(|ui| {
                    if ui.button("Generate").clicked() {
                        let blacklist = if self.privacy_mode {
                            generate_blacklist(&self.entries)
                        } else {
                            vec![]
                        };

                        let entries_iter = self.entries.iter().chain(blacklist.iter());

                        self.generated = format!(
                            "{}\n{}",
                            "# Generated by jameender/sandboxie-whitelist",
                            &entries_iter
                                .map(|e| {
                                    let path = std::path::Path::new(&e.path);

                                    format!(
                                        "{}={}{}",
                                        e.access.to_ini_string(),
                                        e.path,
                                        if path.is_dir() { "\\*" } else { "" }
                                    )
                                })
                                .collect::<Vec<String>>()
                                .join("\n"),
                        );
                    }
                    if !self.generated.is_empty() && ui.button("Copy").clicked() {
                        ui.output_mut(|o| o.copied_text = self.generated.clone());
                    }
                });

                if !self.generated.is_empty() {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.generated)
                            .desired_width(ui.available_width()),
                    );
                }
            });
        });
    }
}

fn generate_blacklist(sandbox_entries: &[Entry]) -> Vec<Entry> {
    let mut blacklist = Vec::new();
    let mut already_seen = std::collections::HashSet::new();

    for sandbox_entry in sandbox_entries
        .into_iter()
        .filter(|e| e.access != Access::BoxOnly && e.access != Access::Closed)
    {
        let path = std::path::Path::new(&sandbox_entry.path);
        let components: Vec<_> = path.components().collect();
        let mut current_path = std::path::PathBuf::new();

        for (i, component) in components.iter().enumerate() {
            current_path.push(component);

            if i == 0 {
                current_path.push("\\");
            }

            if component.as_os_str() == "."
                || component.as_os_str() == ".."
                || i + 1 >= components.len()
            {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&current_path) {
                for entry in entries.filter_map(Result::ok) {
                    let next_component = components[i + 1];

                    if entry.file_name().as_os_str() == next_component.as_os_str() {
                        continue;
                    }

                    if sandbox_entries
                        .iter()
                        .any(|e| std::path::Path::new(&e.path).starts_with(entry.path()))
                    {
                        continue;
                    }

                    let path = std::path::Path::new(&current_path);

                    let path = path
                        .join(entry.file_name())
                        .to_string_lossy()
                        .into_owned()
                        .replace("\\\\?\\", "");

                    if already_seen.contains(&path) {
                        continue;
                    }

                    already_seen.insert(path.clone());

                    blacklist.push(Entry {
                        path,
                        access: Access::BoxOnly,
                    });
                }
            }
        }
    }

    let does_contain_users = sandbox_entries.iter().any(|e| e.path.contains("C:\\Users"));

    // For some unknown reason to me this is needed to prevent the users folder from being blacklisted
    if does_contain_users {
        blacklist
            .into_iter()
            .filter(|e| !e.path.contains("C:\\Documents and Settings"))
            .collect()
    } else {
        blacklist
    }
}
