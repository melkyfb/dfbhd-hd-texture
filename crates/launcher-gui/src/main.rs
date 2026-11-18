mod app;
mod model;
mod theme;

use app::LauncherApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "MELKY.LAUNCHER // SECURE_VAULT",
        options,
        Box::new(|_cc| Ok(Box::new(LauncherApp::default()))),
    )
}

