use bevy::prelude::*;
use crate::ui::fonts::setup_fonts;

/// Plugin that handles all UI setup including fonts and styling
pub struct UISetupPlugin;

impl Plugin for UISetupPlugin {
    fn build(&self, app: &mut App) {
        // Add the font setup system to run at startup
        app.add_systems(Startup, setup_fonts);
    }
}
