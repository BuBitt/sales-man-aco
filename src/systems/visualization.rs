use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use crate::components::*;
use crate::resources::*;
use crate::utils::{safe_despawn, safe_despawn_collection};

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
        
        for i in 0..n {
            for j in (i+1)..n {
                let dist = points.positions[i].distance(points.positions[j]);
                distances[i][j] = dist;
                distances[j][i] = dist;
            }
        }
        
        distance_matrix.distances = distances;
    }
}

pub fn update_candidate_lists(
    points: Res<Points>,
    distance_matrix: Res<DistanceMatrix>,
    mut candidate_lists: ResMut<CandidateList>,
) {
    if points.is_changed() && !points.positions.is_empty() && !distance_matrix.distances.is_empty() {
        let n = points.positions.len();
        
        // Safety check - need at least 2 points to create candidate lists
        if n < 2 {
            candidate_lists.nearest_neighbors = Vec::new();
            return;
        }
        
        let mut nearest_neighbors = vec![Vec::with_capacity(crate::constants::CANDIDATE_LIST_SIZE.min(n-1)); n];
        
        for i in 0..n {
            // Create a vector of (index, distance) pairs - skip self connections (j != i)
            let mut distances: Vec<(usize, f32)> = (0..n)
                .filter(|&j| j != i)
                .map(|j| (j, distance_matrix.distances[i][j]))
                .collect();
                
            // Sort by distance (shortest first)
            distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            
            // Fix: Make sure we don't take more elements than are available
            let available = distances.len();
            let to_take = crate::constants::CANDIDATE_LIST_SIZE.min(available);
            
            // Fix: Use first element of tuple directly without pattern matching
            nearest_neighbors[i] = distances
                .into_iter()
                .take(to_take)
                .map(|(idx, _)| idx)
                .collect();
        }
        
        candidate_lists.nearest_neighbors = nearest_neighbors;
    }
}
