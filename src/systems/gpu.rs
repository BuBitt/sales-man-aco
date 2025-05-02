use bevy::prelude::*;
use crate::resources::*;

/// Sistema para sincronizar operações entre GPU e CPU
pub fn gpu_sync_system(
    mut gpu_sync: ResMut<GpuSyncState>,
    aco_state: ResMut<AcoState>,
    points: Res<Points>,
    gpu_config: Res<GpuConfig>,
) {
    // Só gerencia sincronização para grandes conjuntos de pontos
    if !gpu_config.enabled || points.positions.len() <= gpu_config.use_gpu_threshold {
        // Para conjuntos pequenos, sempre permitir execução
        gpu_sync.computation_completed = true;
        gpu_sync.iteration_ready = true;
        return;
    }

    // Corrigir o problema de iteração limitada para grandes conjuntos
    if aco_state.running && aco_state.iterations > gpu_sync.last_processed_iteration {
        // Rastrear a iteração atual
        gpu_sync.last_processed_iteration = aco_state.iterations;
        
        // Evitar bloqueio de execução acidental
        if gpu_sync.frame_skip_counter >= 2 {  // Pule alguns frames para dar tempo à GPU
            gpu_sync.computation_completed = true;
            gpu_sync.iteration_ready = true;
            gpu_sync.frame_skip_counter = 0;
        } else {
            gpu_sync.frame_skip_counter += 1;
        }
    }
    
    // Garantir que o algoritmo sempre possa avançar
    if !gpu_sync.iteration_ready && aco_state.running {
        gpu_sync.iteration_ready = true;
    }
}
