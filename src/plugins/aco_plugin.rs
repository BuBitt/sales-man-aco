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
                    .run_if(|
                        state: Res<AcoState>, 
                        points: Res<Points>,
                        gpu_sync: Res<GpuSyncState>,
                        gpu_config: Res<GpuConfig>
                    | {
                        if !state.running {
                            return false;
                        }
                        
                        let point_count = points.positions.len();
                        
                        // Para problemas grandes, verifica sincronização GPU
                        if point_count > gpu_config.use_gpu_threshold {
                            // Só executa se a sincronização GPU permitir
                            state.running && gpu_sync.iteration_ready
                        } else if point_count > constants::PARALLEL_THRESHOLD {
                            // Para problemas médios, mantém a lógica existente
                            state.running && (state.iterations == 0 || state.iterations % 3 == 0)
                        } else {
                            // Para problemas pequenos, executa normalmente
                            state.running
                        }
                    })
            );
    }
}
