mod model;
mod theme;

use eframe::egui;

#[derive(Default)]
struct VaultShellApp;

impl eframe::App for VaultShellApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label("SYSTEM_STATE: READY");
        });
    }
}



fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "MELKY.LAUNCHER // SECURE_VAULT",
        options,
        Box::new(|_cc| Ok(Box::new(VaultShellApp::default()))),
    )
}

