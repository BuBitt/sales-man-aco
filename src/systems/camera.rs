use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use crate::components::*;
use crate::resources::UiState;

pub fn camera_drag(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(Entity, &mut Transform, Option<&Dragging>), With<MainCamera>>,
    ui_state: Res<UiState>,
) {
    let (camera_entity, mut camera_transform, dragging) = query.single_mut();

    if mouse_buttons.just_pressed(MouseButton::Left) && !ui_state.interacting_with_ui {
        commands.entity(camera_entity).insert(Dragging);
    }

    if mouse_buttons.just_released(MouseButton::Left) || ui_state.interacting_with_ui {
        if dragging.is_some() {
            commands.entity(camera_entity).remove::<Dragging>();
        }
    }

    if dragging.is_some() && !ui_state.interacting_with_ui {
        for event in mouse_motion_events.read() {
            camera_transform.translation.x -= event.delta.x * camera_transform.scale.x;
            camera_transform.translation.y += event.delta.y * camera_transform.scale.y;
        }
    }
}

pub fn camera_zoom(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<MainCamera>>,
    ui_state: Res<UiState>,
) {
    if ui_state.interacting_with_ui {
        return;
    }

    let mut camera_transform = query.single_mut();

    for event in mouse_wheel_events.read() {
        let zoom_factor = if event.y > 0.0 { 0.9 } else { 1.1 };

        camera_transform.scale.x = (camera_transform.scale.x * zoom_factor).clamp(0.1, 5.0);
        camera_transform.scale.y = (camera_transform.scale.y * zoom_factor).clamp(0.1, 5.0);
    }
}
