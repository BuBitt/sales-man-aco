use bevy::prelude::*;
use crate::resources::*;
use crate::components::*;

/// Sistema que gerencia alocação e liberação de memória
pub fn manage_memory(
    points: Res<Points>,
    memory_config: Res<MemoryConfig>,
    _vector_pool: ResMut<VectorPool>,
    _commands: Commands,
    mut camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    visible_entities: Query<(Entity, &GlobalTransform), With<PointMarker>>,
) {
    // Limites inteligentes de objetos visíveis
    if points.positions.len() > memory_config.max_points_in_view {
        if let Ok((camera, camera_transform)) = camera_query.get_single_mut() {
            // Determina quais pontos estão dentro do viewport da câmera
            if let Some(viewport_rect) = camera.logical_viewport_rect() {
                let visible_count = visible_entities
                    .iter()
                    .filter(|(_, transform)| {
                        // Verifica se o ponto está dentro da visualização da câmera
                        let pos = transform.translation();
                        if let Some(view_pos) = camera.world_to_viewport(camera_transform, pos) {
                            viewport_rect.contains(view_pos)
                        } else {
                            false
                        }
                    })
                    .count();
                
                // Limita o número de elementos visíveis para evitar sobrecarga
                if visible_count > memory_config.max_points_in_view {
                    info!("Limitando visualização para {} pontos de {}", 
                          memory_config.max_points_in_view, visible_count);
                }
            }
        }
    }
}
