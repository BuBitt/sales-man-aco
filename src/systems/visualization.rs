use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use crate::components::*;
use crate::resources::*;
use crate::utils::{safe_despawn, safe_despawn_collection};
// Import all required Rayon traits
use rayon::prelude::*;

pub fn update_points_visualization(
    mut commands: Commands,
    points: Res<Points>,
    mut entity_tracker: ResMut<EntityTracker>,
    point_markers: Query<Entity, With<PointMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if (!points.is_changed() && !point_markers.is_empty() && !entity_tracker.point_entities.is_empty()) 
        || points.positions.is_empty() {
        return;
    }

    for entity in point_markers.iter() {
        safe_despawn(&mut commands, entity);
    }

    entity_tracker.point_entities.clear();

    let circle = meshes.add(Circle::new(5.0));
    let material = materials.add(ColorMaterial::from(Color::WHITE));

    for position in &points.positions {
        let entity = commands.spawn((
            MaterialMesh2dBundle {
                mesh: circle.clone().into(),
                material: material.clone(),
                transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
                ..default()
            },
            PointMarker,
        )).id();
        
        entity_tracker.point_entities.push(entity);
    }
}

pub fn update_path_visualization(
    mut commands: Commands,
    best_path: Res<BestPath>,
    points: Res<Points>,
    mut entity_tracker: ResMut<EntityTracker>,
    best_path_lines: Query<Entity, With<BestPathLine>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !best_path.is_changed() || best_path.path.is_empty() {
        return;
    }

    safe_despawn_collection(&mut commands, &mut entity_tracker.path_entities);

    for entity in best_path_lines.iter() {
        safe_despawn(&mut commands, entity);
    }

    let n = best_path.path.len();
    let _line_material = materials.add(ColorMaterial::from(Color::srgb(0.0, 1.0, 0.0)));

    entity_tracker.path_entities.clear();
    for i in 0..n {
        let from_idx = best_path.path[i];
        let to_idx = best_path.path[(i + 1) % n];

        if from_idx >= points.positions.len() || to_idx >= points.positions.len() {
            continue;
        }

        let from = points.positions[from_idx];
        let to = points.positions[to_idx];
        let direction = to - from;
        let length = direction.length();

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    custom_size: Some(Vec2::new(length, 1.5)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0, 0.0),
                    rotation: Quat::from_rotation_z(direction.y.atan2(direction.x)),
                    ..default()
                },
                ..default()
            },
            BestPathLine,
        )).id();
        
        entity_tracker.path_entities.push(entity);
    }
}

pub fn update_distance_matrix(
    points: Res<Points>,
    mut distance_matrix: ResMut<DistanceMatrix>,
) {
    if points.is_changed() && !points.positions.is_empty() {
        let n = points.positions.len();
        let mut distances = vec![vec![0.0; n]; n];
        
        // Use the correct Rayon parallel iteration pattern
        if n > 100 {
            distances.par_iter_mut().enumerate().for_each(|(i, row)| {
                for j in (i+1)..n {
                    let dist = points.positions[i].distance(points.positions[j]);
                    row[j] = dist;
                    // Note: we can't set distances[j][i] here because of borrow rules
                }
            });
            
            // Fill the other half of the matrix in a second pass
            for i in 0..n {
                for j in (i+1)..n {
                    distances[j][i] = distances[i][j];
                }
            }
        } else {
            // Sequential implementation for small datasets
            for i in 0..n {
                for j in (i+1)..n {
                    let dist = points.positions[i].distance(points.positions[j]);
                    distances[i][j] = dist;
                    distances[j][i] = dist;
                }
            }
        }
        
        distance_matrix.distances = distances;
    }
}

pub fn update_candidate_lists(
    points: Res<Points>,
    distances: Res<DistanceMatrix>,
    mut candidate_lists: ResMut<CandidateList>,
) {
    let n = points.positions.len();
    
    // Se não houver pontos, limpa a lista de candidatos e retorna
    if n == 0 {
        candidate_lists.nearest_neighbors = Vec::new();
        return;
    }
    
    // Verificar se a matriz de distância está inicializada corretamente
    if distances.distances.is_empty() {
        candidate_lists.nearest_neighbors = Vec::new();
        return;
    }
    
    // Garantir que a matriz de distâncias tem o tamanho correto
    if distances.distances.len() != n {
        candidate_lists.nearest_neighbors = Vec::new();
        return;
    }
    
    // Para cada linha na matriz, verifique se ela tem o comprimento correto
    for row in &distances.distances {
        if row.len() != n {
            candidate_lists.nearest_neighbors = Vec::new();
            return;
        }
    }

    // Máximo de candidatos é 20 ou n-1, o que for menor
    let k = usize::min(crate::constants::CANDIDATE_LIST_SIZE, n.saturating_sub(1));
    
    // Se n <= 1, não há candidatos a serem calculados
    if n <= 1 {
        candidate_lists.nearest_neighbors = vec![Vec::new(); n];
        return;
    }
    
    // Pré-aloca o vetor com a capacidade exata para evitar realocações
    let mut new_lists = Vec::with_capacity(n);
    
    for i in 0..n {
        // Coletar todos os vizinhos válidos
        let mut neighbors = Vec::with_capacity(n - 1);
        
        for j in 0..n {
            if i != j {
                neighbors.push((j, distances.distances[i][j]));
            }
        }
        
        // Ordenar por distância
        neighbors.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Extrair apenas os k vizinhos mais próximos
        let closest = neighbors.iter()
            .take(k)
            .map(|&(j, _)| j)
            .collect();
            
        new_lists.push(closest);
    }
    
    candidate_lists.nearest_neighbors = new_lists;
}
