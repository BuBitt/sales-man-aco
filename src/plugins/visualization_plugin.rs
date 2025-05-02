use bevy::prelude::*;
use crate::resources::*;
use crate::systems::*;

pub struct VisualizationPlugin;

impl Plugin for VisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            // UI system - executa em cada frame
            .add_systems(Update, ui_system)
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
            );
    }
}
