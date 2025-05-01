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
    distances: Res<DistanceMatrix>,
    mut candidate_lists: ResMut<CandidateList>,
) {
    let n = points.positions.len();
    
    // If there are no points, reset the candidate list to be empty
    if n == 0 {
        candidate_lists.nearest_neighbors = Vec::new();
        return;
    }

    // Maximum number of candidates is either 20 or n-1, whichever is smaller
    let k = usize::min(20, n.saturating_sub(1)); // Use saturating_sub to handle the case where n might be 0
    
    // For each point, find its k nearest neighbors
    candidate_lists.nearest_neighbors = (0..n)
        .map(|i| {
            // Skip points with no neighbors (shouldn't happen with our check above, but just to be safe)
            if n <= 1 {
                return Vec::new();
            }
            
            // For each point, get distances to all other points
            let mut neighbors: Vec<(usize, f32)> = (0..n)
                .filter(|&j| i != j)
                .map(|j| (j, distances.distances[i][j]))
                .collect();
            
            // Sort by distance
            neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            
            // Take only the k closest neighbors
            neighbors.iter()
                .take(k)
                .map(|&(j, _)| j)
                .collect()
        })
        .collect();
}
