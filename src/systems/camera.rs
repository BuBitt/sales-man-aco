use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::input::mouse::MouseMotion;
use crate::components::MainCamera;
use crate::resources::UiState;

/// Sistema para controlar zoom da câmera com roda do mouse
pub fn camera_zoom(
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    ui_state: Res<UiState>,
) {
    // Skip ALL processing if mouse is over UI or interacting with UI
    // This is the key fix - immediately return if hovering UI
    if ui_state.hovering_ui {
        // Clear all events to prevent them from being processed later
        mouse_wheel.clear();
        return;
    }
    
    // Process zoom events only if not over UI
    for event in mouse_wheel.read() {
        if let Ok(mut projection) = camera_query.get_single_mut() {
            // Apply zoom
            projection.scale += event.y * 0.1 * projection.scale;
            
            // Clamp to reasonable values
            projection.scale = projection.scale.clamp(0.1, 10.0);
        }
    }
}

/// Sistema para arrastar a câmera com o mouse
pub fn camera_drag(
    mut mouse_motion_events: EventReader<MouseMotion>,
    _mouse_buttons: Res<ButtonInput<MouseButton>>,
    _camera_query: Query<&mut Transform, With<MainCamera>>,
    ui_state: Res<UiState>,
) {
    // Don't move camera if interacting with UI
    if ui_state.interacting_with_ui || ui_state.hovering_ui {
        mouse_motion_events.clear();
        return;
    }

    // Note: Actual drag implementation would go here
}
