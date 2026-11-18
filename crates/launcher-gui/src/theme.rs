use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultPalette {
    pub bg: egui::Color32,
    pub panel: egui::Color32,
    pub border: egui::Color32,
    pub accent: egui::Color32,
    pub accent_alt: egui::Color32,
    pub text: egui::Color32,
    pub muted: egui::Color32,
}

impl Default for VaultPalette {
    fn default() -> Self {
        Self {
            bg: egui::Color32::from_rgb(10, 14, 18),
            panel: egui::Color32::from_rgb(16, 22, 28),
            border: egui::Color32::from_rgb(38, 70, 78),
            accent: egui::Color32::from_rgb(0, 210, 255),      // cyan
            accent_alt: egui::Color32::from_rgb(0, 255, 170),  // green
            text: egui::Color32::from_rgb(220, 235, 240),
            muted: egui::Color32::from_rgb(120, 145, 155),
        }
    }
}

pub fn palette() -> VaultPalette {
    VaultPalette::default()
}

pub fn apply_vault_theme(ctx: &egui::Context) {
    let p = palette();
    let mut visuals = egui::Visuals::dark();

    visuals.override_text_color = Some(p.text);
    visuals.window_fill = p.bg;
    visuals.panel_fill = p.panel;
    visuals.extreme_bg_color = egui::Color32::from_rgb(6, 9, 12);
    visuals.faint_bg_color = p.panel;

    visuals.selection.bg_fill = p.accent;
    visuals.selection.stroke = egui::Stroke::new(1.0, p.accent_alt);

    visuals.widgets.noninteractive.bg_fill = p.panel;
    visuals.widgets.noninteractive.weak_bg_fill = p.panel;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, p.text);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, p.border);

    visuals.widgets.inactive.bg_fill = p.panel;
    visuals.widgets.inactive.weak_bg_fill = p.panel;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, p.muted);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, p.border);

    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(22, 34, 42);
    visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(22, 34, 42);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, p.text);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, p.accent);

    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(18, 46, 54);
    visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(18, 46, 54);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, p.accent_alt);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, p.accent_alt);

    visuals.widgets.open.bg_fill = p.panel;
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, p.text);
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, p.accent);

    ctx.set_visuals(visuals);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_is_dark_with_accents() {
        let p = palette();
        assert!(p.bg.r() < 40 && p.bg.g() < 40 && p.bg.b() < 40);
        assert!(p.accent != p.bg);
        assert!(p.accent_alt != p.bg);
        assert!(p.text.r() > 180 || p.text.g() > 180 || p.text.b() > 180);
    }
}
