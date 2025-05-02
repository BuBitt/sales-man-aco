//! # TSP - Ant Colony Optimization
//! 
//! Este é o ponto de entrada principal para a aplicação de otimização de colônia de formigas
//! para resolver o problema do caixeiro viajante (TSP).
//! 
//! A aplicação usa o framework Bevy para renderização e interface do usuário,
//! combinado com algoritmos de otimização implementados em Rust.

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
use plugins::aco_plugin::AcoPlugin;
use plugins::visualization_plugin::VisualizationPlugin;

/// Macro utilitária para formatação de strings
/// 
/// Esta macro é uma forma abreviada para chamar `format!`
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
                })
                .set(bevy::log::LogPlugin {
                    filter: "wgpu=error,bevy_render=info,bevy_ecs=info,ts=debug".to_string(),
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
            EguiPlugin,
            UISetupPlugin,
            AcoPlugin,
            VisualizationPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        // Inicializa os recursos do aplicativo
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
        .add_systems(Update, (handle_input, camera_drag, camera_zoom))
        .run();
}

/// Configura a cena inicial e a câmera
fn setup(
    mut commands: Commands,
) {
    // Configuração da câmera principal com zoom out e deslocamento para a direita
    commands.spawn((
        Camera2dBundle {
            transform: Transform {
                translation: Vec3::new(150.0, 0.0, 0.0), // Deslocamento para a direita
                scale: Vec3::new(1.2, 1.2, 1.0),         // Pequeno zoom out
                ..default()
            },
            ..default()
        },
        MainCamera,
    ));

    // Texto de instrução
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
    
    info!("Aplicação inicializada com sucesso");
}
