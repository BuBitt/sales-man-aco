use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod constants;
mod components;
mod resources;
mod aco;
mod systems;
mod utils;
mod ui;
mod plugins;

use components::*;
use resources::*;
use systems::*;
use plugins::ui_plugin::UISetupPlugin;

#[macro_export]
macro_rules! format_str {
    ($format:expr, $($arg:expr),*) => {{
        format!($format, $($arg),*)
    }};
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "TSP - Ant Colony Optimization".into(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                }),
            EguiPlugin,
            UISetupPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .insert_resource(AcoState::default())
        .insert_resource(Points::new())
        .insert_resource(BestPath::default())
        .insert_resource(AcoParameters::default())
        .insert_resource(UiState::default())
        .insert_resource(EntityTracker::default())
        .insert_resource(DistanceMatrix::default())
        .insert_resource(CandidateList::default())
        .insert_resource(AppLanguage::default())
        .add_systems(Startup, setup)
        .add_systems(Update, ui_system)
        .add_systems(Update, handle_input)
        .add_systems(Update, camera_drag)
        .add_systems(Update, camera_zoom)
        .add_systems(Update, update_distance_matrix) // Run this first
        .add_systems(Update, update_candidate_lists) // Then run this
        .add_systems(Update, update_points_visualization)
        .add_systems(Update, update_path_visualization)
        .add_systems(Update, run_aco_algorithm)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
    ));

    commands.spawn(
        TextBundle::from_section(
            "Drag to pan, scroll to zoom",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}
