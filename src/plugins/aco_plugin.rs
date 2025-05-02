use bevy::prelude::*;
use crate::resources::*;
use crate::systems::*;
use crate::constants;

pub struct AcoPlugin;

impl Plugin for AcoPlugin {
    fn build(&self, app: &mut App) {
        app
            // Sistemas de cálculo intensivo - executam apenas quando os pontos mudam
            .add_systems(
                Update,
                (update_distance_matrix, update_candidate_lists)
                    .run_if(resource_changed::<Points>)
            )
            // Sistema ACO - otimizado para diferentes tamanhos de problema
            .add_systems(
                Update, 
                run_aco_algorithm
                    .run_if(|state: Res<AcoState>, points: Res<Points>| {
                        if points.positions.len() > constants::PARALLEL_THRESHOLD {
                            // Para problemas grandes, ajustamos a frequência mas garantimos execução contínua
                            state.running && (state.iterations == 0 || state.iterations % 3 == 0)
                        } else {
                            state.running
                        }
                    })
            );
    }
}
