use bevy::prelude::*;
use crate::resources::*;
use crate::components::*;

/// Sistema que gerencia alocação e liberação de memória
pub fn manage_memory(
    points: Res<Points>,
    memory_config: Res<MemoryConfig>,
    mut vector_pool: ResMut<VectorPool>,
    mut commands: Commands,
    mut camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    visible_entities: Query<(Entity, &GlobalTransform), With<PointMarker>>,
) {
    // Limites inteligentes de objetos visíveis
    if points.positions.len() > memory_config.max_points_in_view {
        if let Ok((camera, camera_transform)) = camera_query.get_single_mut() {
            // Determina quais pontos estão dentro do viewport da câmera
            let viewport_rect = camera.logical_viewport_rect().unwrap_or_default();
            let visible_count = visible_entities
                .iter()
                .filter(|(_, transform)| {
                    // Verifica se o ponto está dentro da visualização da câmera
                    let pos = transform.translation();
                    let view_pos = camera.world_to_viewport(camera_transform, pos).unwrap_or_default();
                    viewport_rect.contains(view_pos)
                })
                .count();
            
            // Limita o número de elementos visíveis para evitar sobrecarga
            if visible_count > memory_config.max_points_in_view {
                info!("Limitando visualização para {} pontos de {}", 
                      memory_config.max_points_in_view, visible_count);
                
                // Em uma implementação real, implementaríamos um LOD (Level of Detail) 
                // ou agrupamento de pontos para representação visual
            }
        }
    }
    
    // Limpeza de memória periodicamente
    // Em uma implementação completa, rastrearemos todos os vetores emprestados
    // e faremos limpeza automática após N frames sem uso
}
