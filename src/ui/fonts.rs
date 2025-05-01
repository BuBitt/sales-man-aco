use bevy_egui::{egui, EguiContexts};

/// Sets up custom fonts for the UI
/// This system runs at startup to configure the egui context
pub fn setup_fonts(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    
    // Clone the style so we can modify it
    let mut style = (*ctx.style()).clone();
    
    // Set larger font sizes for better readability
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::proportional(24.0)),
        (egui::TextStyle::Body, egui::FontId::proportional(16.0)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(14.0)),
        (egui::TextStyle::Button, egui::FontId::proportional(16.0)),
        (egui::TextStyle::Small, egui::FontId::proportional(12.0)),
    ].into();
    
    // Improve visuals
    let mut visuals = style.visuals.clone();
    visuals.window_rounding = egui::Rounding::same(6.0);
    
    // Fix the shadow properties using the correct fields
    visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::Vec2::new(2.0, 2.0),
        blur: 10.0,
        spread: 2.0,
        color: egui::Color32::from_rgba_premultiplied(0, 0, 0, 60),
    };
    
    visuals.button_frame = true;
    visuals.widgets.noninteractive.rounding = egui::Rounding::same(2.0);
    visuals.widgets.inactive.rounding = egui::Rounding::same(2.0);
    visuals.widgets.hovered.rounding = egui::Rounding::same(3.0);
    visuals.widgets.active.rounding = egui::Rounding::same(2.0);
    
    // Assign the improved visuals to our style copy
    style.visuals = visuals;
    
    // Set the entire style object at once
    ctx.set_style(style);
}
