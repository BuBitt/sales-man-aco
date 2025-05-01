use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use crate::components::MainCamera;
use crate::resources::UiState;

pub fn camera_zoom(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut OrthographicProjection, With<MainCamera>>,
    ui_state: Res<UiState>,
) {
    if ui_state.interacting_with_ui {
        return;
    }

    let zoom_amount = mouse_wheel_events
        .read()
        .fold(0.0, |acc, ev| acc + ev.y);

    if zoom_amount == 0.0 {
        return;
    }

    let mut projection = query.single_mut();
    
    // Increase max zoom out to see all points
    let min_scale = 0.1; // More zoom out
    let max_scale = 2.0; // More zoom in
    
    // Smoother zoom with exponential scale
    projection.scale = (projection.scale * (1.0 - zoom_amount * 0.1))
        .max(min_scale)
        .min(max_scale);
}

pub fn camera_drag(
    mut mouse_motion_events: EventReader<MouseMotion>,
    keyboard: Res<ButtonInput<MouseButton>>, // Updated to use ButtonInput for Bevy 0.14
    mut query: Query<&mut Transform, With<MainCamera>>,
    ui_state: Res<UiState>,
) {
    if ui_state.interacting_with_ui || !keyboard.pressed(MouseButton::Right) {
        return;
    }

    let delta = mouse_motion_events
        .read()
        .fold(Vec2::ZERO, |acc, ev| acc + ev.delta);

    if delta == Vec2::ZERO {
        return;
    }

    let mut transform = query.single_mut();

    // Scale movement based on zoom level for consistent feel
    let zoom_factor = transform.scale.x.max(0.1);
    transform.translation.x -= delta.x * zoom_factor;
    transform.translation.y += delta.y * zoom_factor;
}
