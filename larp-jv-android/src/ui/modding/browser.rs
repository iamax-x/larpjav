//! Mod Browser UI — port of ModVersionListFragment.java

use eframe::egui;
use larp_jv_core::value::{ModInfo, ModLoaderType};
use std::sync::Arc;

pub struct ModBrowserUi {
    mod_manager: Arc<crate::modding::ModManager>,
    installed_mods: Vec<ModInfo>,
    available_mods: Vec<ModInfo>,
    selected_mod: Option<ModInfo>,
    filter_loader: Option<ModLoaderType>,
    search_text: String,
}

impl ModBrowserUi {
    pub fn new(mod_manager: Arc<crate::modding::ModManager>) -> Self {
        Self {
            mod_manager,
            installed_mods: Vec::new(),
            available_mods: Vec::new(),
            selected_mod: None,
            filter_loader: None,
            search_text: String::new(),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::Window::new("Mod Browser")
            .resizable(true)
            .min_width(700.0)
            .min_height(500.0)
            .show(ctx, |ui| {
                ui.heading("Mod Browser");
                ui.separator();
                ui.add_space(8.0);

                // Toolbar
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.search_text).hint_text("Search installed mods...");
                    ui.add(egui::ComboBox::from_label("Filter")
                        .selected_text("All")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut (), (), "All");
                            ui.selectable_value(&mut (), (), "Enabled");
                            ui.selectable_value(&mut (), (), "Disabled");
                        }));
                    if ui.button("Refresh").clicked() {
                        // Refresh list
                    }
                    if ui.button("Get More Mods").clicked() {
                        // Open search window
                    }
                });
                ui.separator();

                // Mod list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Placeholder for installed mods
                    ui.label("No mods installed yet.");
                    ui.add_space(16.0);
                    ui.label("Click 'Get More Mods' to browse Modrinth/CurseForge");
                });
            });
    }
}