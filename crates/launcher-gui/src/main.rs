mod app;
mod config;
mod detect;
mod model;
mod theme;

use app::LauncherApp;

fn main() -> eframe::Result<()> {
    let cfg = config::load_config();
    let mut state = model::LauncherState::default();
    state.apply_loaded_config(&cfg);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "MELKY.LAUNCHER // SECURE_VAULT",
        options,
        Box::new(|_cc| Ok(Box::new(LauncherApp::with_state(state)))),
    )
}
