//! Launcher Activity — Main entry point UI (egui)

use eframe::egui;
use larp_jv_core::{AuthManager, Downloader, InstanceManager, ModManager, JreManager, MoJsonManager};
use std::sync::Arc;

/// Launcher UI state
pub struct LauncherUi {
    auth_manager: Arc<AuthManager>,
    downloader: Arc<Downloader>,
    instance_manager: Arc<InstanceManager>,
    mod_manager: Arc<ModManager>,
    jre_manager: Arc<JreManager>,
    mojson_manager: Arc<MoJsonManager>,
    current_screen: Screen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    MainMenu,
    Instances,
    Settings,
    Modding,
    Auth,
}

impl LauncherUi {
    pub fn new(
        auth_manager: Arc<AuthManager>,
        downloader: Arc<Downloader>,
        instance_manager: Arc<InstanceManager>,
        mod_manager: Arc<ModManager>,
        jre_manager: Arc<JreManager>,
        mojson_manager: Arc<MoJsonManager>,
    ) -> Self {
        Self {
            auth_manager,
            downloader,
            instance_manager,
            mod_manager,
            jre_manager,
            mojson_manager,
            current_screen: Screen::MainMenu,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        // Top bar
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("larp.jv");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Settings").clicked() {
                        self.current_screen = Screen::Settings;
                    }
                    if ui.button("Instances").clicked() {
                        self.current_screen = Screen::Instances;
                    }
                    if ui.button("Mods").clicked() {
                        self.current_screen = Screen::Modding;
                    }
                });
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_screen {
                Screen::MainMenu => self.show_main_menu(ui),
                Screen::Instances => self.show_instances(ui),
                Screen::Settings => self.show_settings(ui),
                Screen::Modding => self.show_modding(ui),
                Screen::Auth => self.show_auth(ui),
            }
        });
    }

    fn show_main_menu(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("Welcome to larp.jv");
            ui.add_space(20.0);
            ui.label("Pure Rust Minecraft Java Edition Launcher");
            ui.add_space(30.0);
            
            if ui.add(egui::Button::new("Play").min_size(egui::vec2(200.0, 50.0))).clicked() {
                // Launch selected instance
            }
            ui.add_space(10.0);
            
            if ui.add(egui::Button::new("Create Instance").min_size(egui::vec2(200.0, 50.0))).clicked() {
                // Navigate to instances
            }
            ui.add_space(10.0);
            
            if ui.add(egui::Button::new("Install Mods").min_size(egui::vec2(200.0, 50.0))).clicked() {
                // Navigate to modding
            }
        });
    }

    fn show_instances(&mut self, ui: &mut egui::Ui) {
        ui.heading("Instances");
        ui.separator();
        
        let instances = self.instance_manager.list_instances();
        
        if instances.is_empty() {
            ui.label("No instances yet. Create one!");
            if ui.button("Create Instance").clicked() {
                // Show create dialog
            }
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for instance in instances {
                    ui.horizontal(|ui| {
                        ui.label(&instance.name);
                        ui.label(&instance.version_id);
                        if ui.button("Play").clicked() {
                            // Launch
                        }
                        if ui.button("Edit").clicked() {
                            // Edit
                        }
                    });
                });
            });
        }
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.collapsing("Java", |ui| {
                ui.label("Java path: auto-detect");
                ui.label("JVM args: -Xmx2G -Xms1G");
            });
            
            ui.collapsing("Video", |ui| {
                ui.label("Resolution: auto");
                ui.label("Fullscreen: false");
                ui.label("VSync: true");
            });
            
            ui.collapsing("Accounts", |ui| {
                let accounts = self.auth_manager.accounts().list_accounts();
                for acc in accounts {
                    ui.horizontal(|ui| {
                        ui.label(&acc.username);
                        ui.label(format!("{:?}", acc.auth_type));
                    });
                }
                if ui.button("Add Microsoft Account").clicked() {
                    // Start auth
                }
            });
        });
    }

    fn show_modding(&mut self, ui: &mut egui::Ui) {
        ui.heading("Mods & Modpacks");
        ui.separator();
        
        ui.horizontal(|ui| {
            if ui.button("Forge").clicked() {}
            if ui.button("Fabric").clicked() {}
            if ui.button("Quilt").clicked() {}
            if ui.button("NeoForge").clicked() {}
            if ui.button("OptiFine").clicked() {}
        });
        
        ui.separator();
        
        if ui.button("Install Modpack (.mrpack)").clicked() {
            // Open file picker
        }
        
        ui.separator();
        ui.label("Mod Search");
        ui.text_edit_singleline(&mut String::new()); // search box
    }

    fn show_auth(&mut self, ui: &mut egui::Ui) {
        ui.heading("Authentication");
        ui.separator();
        
        ui.horizontal(|ui| {
            if ui.button("Microsoft").clicked() {
                // Start Microsoft OAuth
            }
            if ui.button("Ely.by").clicked() {
                // Start Ely.by
            }
            if ui.button("Offline").clicked() {
                // Create offline account
            }
        });
    }
}