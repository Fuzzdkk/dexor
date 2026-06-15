//! Look + feel lifted from 0xfuzz.com: near-black slate backgrounds, cyan
//! accent, JetBrains Mono everywhere for that terminal vibe.

use eframe::egui::{self, Color32, FontFamily, FontId, Rounding, Stroke};

// palette pulled straight off the site's css.
const fn hex(c: u32) -> Color32 {
    Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

pub const BG0: Color32 = hex(0x0a0d12); // deepest, central panel
pub const BG1: Color32 = hex(0x0d1117); // panels / top+bottom bars
pub const BG2: Color32 = hex(0x161922); // drop zone / text fields
pub const SURFACE: Color32 = hex(0x1c1f2e); // buttons at rest
pub const SURFACE_HOVER: Color32 = hex(0x232739);
pub const SURFACE_ACTIVE: Color32 = hex(0x2a3650);
pub const BORDER: Color32 = hex(0x1e293b);
pub const TEXT: Color32 = hex(0xe2e8f0);
pub const TEXT_STRONG: Color32 = hex(0xf8fafc);
pub const MUTED: Color32 = hex(0x64748b);
pub const ACCENT: Color32 = hex(0x06b6d4); // cyan-500
pub const ACCENT_BRIGHT: Color32 = hex(0x22d3ee); // cyan-400
pub const OK: Color32 = hex(0x4ade80); // green
pub const ERR: Color32 = hex(0xff6b6b); // red
pub const WARN: Color32 = hex(0xfbbf24); // amber

/// A named font family for the bold weight, used by the title.
pub fn bold_font() -> FontFamily {
    FontFamily::Name("jbmono-bold".into())
}

pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);
    install_visuals(ctx);
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "jbmono".into(),
        egui::FontData::from_static(include_bytes!("../assets/JetBrainsMono-Regular.ttf")),
    );
    fonts.font_data.insert(
        "jbmono-bold".into(),
        egui::FontData::from_static(include_bytes!("../assets/JetBrainsMono-Bold.ttf")),
    );

    // JetBrains Mono leads both the proportional and monospace stacks, so the
    // whole UI reads like a terminal (matches the site).
    for fam in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.entry(fam).or_default().insert(0, "jbmono".into());
    }
    // dedicated bold family for the heading.
    fonts
        .families
        .insert(bold_font(), vec!["jbmono-bold".into(), "jbmono".into()]);

    ctx.set_fonts(fonts);
}

fn install_visuals(ctx: &egui::Context) {
    use egui::TextStyle::*;

    let mut style = (*ctx.style()).clone();

    // type scale — a touch larger + roomier than egui defaults.
    style.text_styles = [
        (Heading, FontId::new(22.0, FontFamily::Proportional)),
        (Body, FontId::new(14.0, FontFamily::Proportional)),
        (Monospace, FontId::new(13.5, FontFamily::Monospace)),
        (Button, FontId::new(14.0, FontFamily::Proportional)),
        (Small, FontId::new(11.5, FontFamily::Proportional)),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(14.0);

    let v = &mut style.visuals;
    v.dark_mode = true;
    v.panel_fill = BG0;
    v.window_fill = BG1;
    v.faint_bg_color = BG1;
    v.extreme_bg_color = BG2;
    v.override_text_color = Some(TEXT);
    v.hyperlink_color = ACCENT_BRIGHT;
    v.window_rounding = Rounding::same(10.0);
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.selection.bg_fill = ACCENT.linear_multiply(0.35);
    v.selection.stroke = Stroke::new(1.0, ACCENT_BRIGHT);

    let r = Rounding::same(8.0);
    let w = &mut v.widgets;

    // non-interactive (labels, separators, frames)
    w.noninteractive.bg_fill = BG1;
    w.noninteractive.weak_bg_fill = BG1;
    w.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    w.noninteractive.rounding = r;

    // buttons / fields at rest
    w.inactive.bg_fill = SURFACE;
    w.inactive.weak_bg_fill = SURFACE;
    w.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    w.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    w.inactive.rounding = r;

    // hover — cyan edge lights up
    w.hovered.bg_fill = SURFACE_HOVER;
    w.hovered.weak_bg_fill = SURFACE_HOVER;
    w.hovered.bg_stroke = Stroke::new(1.0, ACCENT_BRIGHT);
    w.hovered.fg_stroke = Stroke::new(1.5, TEXT_STRONG);
    w.hovered.rounding = r;

    // pressed / active
    w.active.bg_fill = SURFACE_ACTIVE;
    w.active.weak_bg_fill = SURFACE_ACTIVE;
    w.active.bg_stroke = Stroke::new(1.0, ACCENT);
    w.active.fg_stroke = Stroke::new(1.5, TEXT_STRONG);
    w.active.rounding = r;

    // open combo box
    w.open.bg_fill = SURFACE;
    w.open.weak_bg_fill = SURFACE;
    w.open.bg_stroke = Stroke::new(1.0, ACCENT);
    w.open.fg_stroke = Stroke::new(1.0, TEXT_STRONG);
    w.open.rounding = r;

    ctx.set_style(style);
}
