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
mod config;

use components::*;
use resources::*;
use systems::*;
use plugins::ui_plugin::UISetupPlugin;
use plugins::aco_plugin::AcoPlugin;
use plugins::visualization_plugin::VisualizationPlugin;
use plugins::gpu_plugin::GpuAccelerationPlugin;
use config::Config;

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
    // Carregar configurações do arquivo
    let config = Config::load();
    
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(config.get_window()),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes_override: Some(config.rendering.watch_for_changes),
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
            GpuAccelerationPlugin,
        ))
        .insert_resource(config.get_clear_color())
        .insert_resource(config.get_memory_config())
        .insert_resource(config.get_gpu_config())
        .insert_resource(AcoState::default())
        .insert_resource(Points::new())
        .insert_resource(BestPath::default())
        .insert_resource(config.get_aco_parameters())
        .insert_resource(UiState::default())
        .insert_resource(EntityTracker::default())
        .insert_resource(DistanceMatrix::default())
        .insert_resource(CandidateList::default())
        .insert_resource(AppLanguage::default())
        .insert_resource(VectorPool::new(config.memory.vector_pool_size))
        .insert_resource(GpuSyncState::default())
        // Add the config as resource with proper trait
        .insert_resource(config)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_input, 
            camera_drag, 
            camera_zoom,
            // Use system labels or system sets instead of direct function references
            manage_memory,
            gpu_sync_system,
            run_aco_algorithm,
        ).chain())
        .run();
}

/// Configura a cena inicial e a câmera
fn setup(
    mut commands: Commands,
) {
    // Spawn da câmera principal
    // 
    // Configura uma câmera 2D com:
    // - Deslocamento para a esquerda para manter a área de visualização
    //   afastada da interface do usuário
    // - Zoom ajustado para melhor visualização do campo de trabalho
    commands.spawn((
        Camera2dBundle {
            transform: Transform {
                translation: Vec3::new(-250.0, 0.0, 0.0), // Valor negativo desloca visualização para direita
                scale: Vec3::new(1.2, 1.2, 1.0),          // Zoom out para melhor enquadramento
                ..default()
            },
            ..default()
        },
        MainCamera,
    ));

    // Texto de instruções para o usuário
    // 
    // Fornece orientação básica sobre controles de navegação
    // Posicionado no canto inferior esquerdo da tela
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
