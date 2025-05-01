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
                }),
            EguiPlugin,
            UISetupPlugin,
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
        // Sistemas de UI e entrada - executam em cada frame
        .add_systems(Update, (ui_system, handle_input))
        // Sistemas de câmera - executam em cada frame
        .add_systems(Update, (camera_drag, camera_zoom))
        // Sistemas de cálculo intensivo - executam apenas quando os pontos mudam
        .add_systems(
            Update,
            (update_distance_matrix, update_candidate_lists)
                .run_if(resource_changed::<Points>)
        )
        // Sistema de visualização de pontos - executa apenas quando os pontos mudam
        .add_systems(
            Update, 
            update_points_visualization
                .run_if(resource_changed::<Points>)
        )
        // Sistema de visualização de caminhos - executa apenas quando o melhor caminho muda
        .add_systems(
            Update,
            update_path_visualization
                .run_if(resource_changed::<BestPath>)
        )
        // Sistema ACO - otimizado para diferentes tamanhos de problema
        .add_systems(
            Update, 
            run_aco_algorithm
                .run_if(|state: Res<AcoState>, points: Res<Points>| {
                    if points.positions.len() > 100 {
                        // Para problemas grandes, executa menos frequentemente para melhor desempenho
                        state.running && state.iterations % 5 == 0
                    } else {
                        state.running
                    }
                })
        )
        .run();
}

/// Configura a cena inicial e a câmera
/// 
/// Esta função é executada apenas uma vez durante a inicialização e configura:
/// - A câmera principal com projeção ortográfica
/// - O texto de instrução para navegação na interface
fn setup(
    mut commands: Commands,
) {
    // Configuração da câmera principal
    commands.spawn((
        Camera2dBundle::default(),
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
}
