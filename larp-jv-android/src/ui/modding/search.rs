//! Mod Search UI — Modrinth & CurseForge

use eframe::egui;
use larp_jv_core::value::{ModInfo, ModLoaderType};
use std::sync::Arc;

pub struct ModSearchUi {
    mod_manager: Arc<crate::modding::ModManager>,
    query: String,
    loader_filter: Option<ModLoaderType>,
    mc_version: String,
    results: Vec<ModInfo>,
    searching: bool,
    page: usize,
    total_hits: u32,
    error: Option<String>,
}

impl ModSearchUi {
    pub fn new(mod_manager: Arc<crate::modding::ModManager>) -> Self {
        Self {
            mod_manager,
            query: String::new(),
            loader_filter: None,
            mc_version: String::new(),
            results: Vec::new(),
            searching: false,
            page: 0,
            total_hits: 0,
            error: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Search Mods")
            .resizable(true)
            .min_width(600.0)
            .min_height(500.0)
            .show(ctx, |ui| {
                ui.heading("Search Mods");
                ui.separator();
                ui.add_space(8.0);

                // Search controls
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.query).hint_text("Search mods...");
                    ui.add(egui::ComboBox::from_label("Loader")
                        .selected_text(self.loader_filter.map(|l| format!("{:?}", l)).unwrap_or("All".to_string()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.loader_filter, None, "All");
                            ui.selectable_value(&mut self.loader_filter, Some(crate::value::ModLoaderType::Forge), "Forge");
                            ui.selectable_value(&mut self.loader_filter, Some(crate::value::ModLoaderType::Fabric), "Fabric");
                            ui.selectable_value(&mut self.loader_filter, Some(crate::value::ModLoaderType::Quilt), "Quilt");
                            ui.selectable_value(&mut self.loader_filter, Some(crate::value::ModLoaderType::NeoForge), "NeoForge");
                        });
                    ui.text_edit_singleline(&mut self.mc_version).hint_text("MC Version");
                });
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    let search_btn = ui.add_enabled(!self.query.is_empty() && !self.mc_version.is_empty(),
                        egui::Button::new("Search").fill(egui::Color32::from_rgb(0, 150, 0)));
                    if search_btn.clicked() {
                        self.search();
                    }
                    if ui.button("Clear").clicked() {
                        self.results.clear();
                        self.query.clear();
                    }
                });
                ui.add_space(8.0);

                if self.searching {
                    ui.add(egui::ProgressBar::new(0.5).show_percentage());
                    ui.label("Searching...");
                }

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.add_space(8.0);
                }

                // Results
                ui.separator();
                ui.horizontal(|ui| {
                    ui.strong(format!("Results: {} (page {})", self.total_hits, self.page + 1));
                    if self.page > 0 && ui.button("Prev").clicked() {
                        self.page -= 1;
                        self.search();
                    }
                    if ui.button("Next").clicked() {
                        self.page += 1;
                        self.search();
                    }
                });

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for mod_info in &self.results {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.strong(&mod_info.id);
                                    ui.label(format!("v{} • {:?}", mod_info.version, mod_info.loader));
                                });
                                if ui.button("Install").clicked() {
                                    // Install mod
                                }
                            });
                        });
                    });
                });

                // Pagination
                ui.horizontal(|ui| {
                    if self.page > 0 && ui.button("◀ Prev").clicked() {
                        self.page -= 1;
                        self.search();
                    }
                    ui.label(format!("Page {}", self.page + 1));
                    if ui.button("Next ▶").clicked() {
                        self.page += 1;
                        self.search();
                    }
                });
            });
    }

    fn search(&mut self) {
        self.searching = true;
        self.error = None;
        
        // In real implementation, this would call ModManager::search_mods
        // For now, just simulate
        self.searching = false;
        self.results = vec![
            crate::value::ModInfo {
                id: "example_mod".to_string(),
                version: "1.0.0".to_string(),
                loader: crate::value::ModLoaderType::Fabric,
                enabled: true,
                url: Some("https://modrinth.com/mod/example".to_string()),
                sha1: None,
            },
        ];
        self.total_hits = 1;
    }
}