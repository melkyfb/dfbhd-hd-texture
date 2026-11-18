use eframe::egui;

use crate::model::{LauncherState, View};
use crate::theme::{apply_vault_theme, palette};

pub struct LauncherApp {
    state: LauncherState,
}

impl Default for LauncherApp {
    fn default() -> Self {
        Self {
            state: LauncherState::default(),
        }
    }
}

impl LauncherApp {
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        let p = palette();
        egui::Panel::top("header").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("MELKY.LAUNCHER // SECURE_VAULT")
                        .monospace()
                        .strong()
                        .color(p.accent),
                );
                ui.separator();

                let dash = self.state.view == View::Dashboard;
                if ui
                    .selectable_label(dash, egui::RichText::new("Dashboard").monospace())
                    .clicked()
                {
                    self.state.view = View::Dashboard;
                }
                let dfbhd = self.state.view == View::Dfbhd;
                if ui
                    .selectable_label(dfbhd, egui::RichText::new("DFBHD").monospace())
                    .clicked()
                {
                    self.state.view = View::Dfbhd;
                }
                let settings = self.state.view == View::Settings;
                if ui
                    .selectable_label(settings, egui::RichText::new("Settings").monospace())
                    .clicked()
                {
                    self.state.view = View::Settings;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("STATUS: READY")
                            .monospace()
                            .color(p.accent_alt),
                    );
                });
            });
            ui.add_space(4.0);
        });
    }

    fn draw_sidebar(&mut self, ui: &mut egui::Ui) {
        let p = palette();
        egui::Panel::left("games_nodes")
            .resizable(true)
            .default_size(220.0)
            .show(ui, |ui| {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("GAMES/NODES")
                        .monospace()
                        .strong()
                        .color(p.accent),
                );
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for idx in 0..self.state.games.len() {
                        let (id, name, source_linked) = {
                            let g = &self.state.games[idx];
                            (g.id.clone(), g.name.clone(), g.source_linked)
                        };
                        let selected = self.state.selected_game_idx == idx;
                        let is_dfbhd = id == "dfbhd";
                        let label = if is_dfbhd {
                            format!("> {name} [DFBHD]")
                        } else if selected {
                            format!("> {name}")
                        } else {
                            format!("  {name}")
                        };
                        let mut text = egui::RichText::new(label).monospace();
                        if selected || is_dfbhd {
                            text = text.color(p.accent);
                        } else if !source_linked {
                            text = text.color(p.muted);
                        }
                        if ui.selectable_label(selected, text).clicked() {
                            self.state.selected_game_idx = idx;
                        }
                    }
                });
            });
    }

    fn draw_dashboard(&mut self, ui: &mut egui::Ui) {
        let p = palette();
        ui.label(
            egui::RichText::new("STATUS OVERVIEW")
                .monospace()
                .strong()
                .color(p.accent),
        );
        ui.separator();
        ui.label(egui::RichText::new("SYSTEM: READY // VAULT ONLINE").monospace());
        if let Some(game) = self.state.selected_game() {
            let link = if game.source_linked { "LINKED" } else { "UNLINKED" };
            ui.label(
                egui::RichText::new(format!("SELECTED NODE: {} [{}]", game.name, link))
                    .monospace(),
            );
            let active = if game.active_profile_id.is_empty() {
                "-".to_string()
            } else {
                game.profiles
                    .iter()
                    .find(|p| p.id == game.active_profile_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| game.active_profile_id.clone())
            };
            ui.label(
                egui::RichText::new(format!("ACTIVE PROFILE: {active}"))
                    .monospace()
                    .color(p.muted),
            );
        }

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("NODES SUMMARY")
                .monospace()
                .strong()
                .color(p.accent),
        );
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("dashboard_nodes")
            .show(ui, |ui| {
                for (idx, game) in self.state.games.iter().enumerate() {
                    let marker = if idx == self.state.selected_game_idx {
                        "*"
                    } else {
                        "-"
                    };
                    let status = if game.source_linked { "linked" } else { "unlinked" };
                    ui.label(
                        egui::RichText::new(format!(
                            "{marker} {} :: {} :: profiles={} :: {}",
                            game.name,
                            game.id,
                            game.profiles.len(),
                            status
                        ))
                        .monospace(),
                    );
                }
            });
    }

    fn draw_dfbhd(&mut self, ui: &mut egui::Ui) {
        let p = palette();
        ui.label(
            egui::RichText::new("DFBHD // TEXTURE PROFILES")
                .monospace()
                .strong()
                .color(p.accent),
        );
        ui.separator();

        let Some(game_idx) = self
            .state
            .games
            .iter()
            .position(|g| g.id == "dfbhd")
        else {
            ui.label(egui::RichText::new("DFBHD node not available.").monospace());
            return;
        };

        // Keep DFBHD interactions focused on the DFBHD node.
        if self.state.selected_game_idx != game_idx {
            self.state.selected_game_idx = game_idx;
        }

        let source_label = self.state.games[game_idx].source_label.clone();
        ui.label(
            egui::RichText::new(format!("SOURCE: {source_label}"))
                .monospace()
                .color(p.muted),
        );
        ui.add_space(6.0);

        ui.label(egui::RichText::new("PROFILES").monospace().strong());
        let profiles: Vec<(String, String)> = self.state.games[game_idx]
            .profiles
            .iter()
            .map(|pr| (pr.id.clone(), pr.name.clone()))
            .collect();
        let active_id = self.state.games[game_idx].active_profile_id.clone();
        ui.horizontal_wrapped(|ui| {
            for (pid, pname) in profiles {
                let active = pid == active_id;
                if ui
                    .selectable_label(active, egui::RichText::new(pname).monospace())
                    .clicked()
                {
                    self.state.select_profile(&pid);
                }
            }
        });

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(egui::RichText::new("SCAN/ATTACH").monospace())
                .clicked()
            {
                self.state.scan_attach_stub();
            }
            if ui
                .button(egui::RichText::new("APPLY PROFILE").monospace())
                .clicked()
            {
                self.state.apply_profile_stub();
            }
            if ui
                .button(egui::RichText::new("LAUNCH").monospace())
                .clicked()
            {
                self.state.launch_stub();
            }
        });

        ui.add_space(8.0);
        ui.label(egui::RichText::new("LOG / STATUS").monospace().strong());
        ui.separator();
        let log_lines = self.state.games[game_idx].log.clone();
        egui::ScrollArea::vertical()
            .id_salt("dfbhd_log")
            .stick_to_bottom(true)
            .max_height(220.0)
            .show(ui, |ui| {
                if log_lines.is_empty() {
                    ui.label(
                        egui::RichText::new("[no log entries]")
                            .monospace()
                            .color(p.muted),
                    );
                } else {
                    for line in log_lines {
                        ui.label(egui::RichText::new(line).monospace());
                    }
                }
            });
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui) {
        let p = palette();
        ui.label(
            egui::RichText::new("SETTINGS // PLACEHOLDERS")
                .monospace()
                .strong()
                .color(p.accent),
        );
        ui.separator();
        ui.add_space(4.0);

        ui.label(egui::RichText::new("Resolution: [placeholder]").monospace());
        ui.label(egui::RichText::new("Theme: Vault (default) [placeholder]").monospace());
        ui.label(egui::RichText::new("App prefs: [placeholder]").monospace());
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("No configurable options in Etapa 1.")
                .monospace()
                .color(p.muted),
        );
    }

    fn draw_central(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| match self.state.view {
                    View::Dashboard => self.draw_dashboard(ui),
                    View::Dfbhd => self.draw_dfbhd(ui),
                    View::Settings => self.draw_settings(ui),
                });
        });
    }
}

impl eframe::App for LauncherApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        apply_vault_theme(ui.ctx());
        self.draw_header(ui);
        self.draw_sidebar(ui);
        self.draw_central(ui);
    }
}
