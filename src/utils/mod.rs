use bevy::prelude::*;
use std::collections::HashMap;

pub mod point_distribution;

/// Safely despawns an entity, only if it exists in the world
pub fn safe_despawn(commands: &mut Commands, entity: Entity) {
    // Use a specialized command to despawn only if entity exists
    commands.add(move |world: &mut World| {
        if world.get_entity(entity).is_some() {
            world.despawn(entity);
        }
    });
}

/// Safely despawns a collection of entities, clearing the collection afterward
pub fn safe_despawn_collection(commands: &mut Commands, entities: &mut Vec<Entity>) {
    if entities.is_empty() {
        return;
    }

    // Make a copy of the entities to avoid borrowing issues
    let entities_to_despawn = entities.clone();
    
    // Use a custom command to despawn only existing entities
    commands.add(move |world: &mut World| {
        for entity in entities_to_despawn {
            if world.get_entity(entity).is_some() {
                world.despawn(entity);
            }
        }
    });
    
    // Clear the collection to avoid future attempts to despawn these entities
    entities.clear();
}

/// Calculate distance between two Vec2 points
pub fn calculate_distance(a: Vec2, b: Vec2) -> f32 {
    a.distance(b)
}

/// Calculate the total distance of a TSP path
pub fn calculate_path_distance(path: &[usize], distances: &[Vec<f32>]) -> f32 {
    let mut total = 0.0;
    let n = path.len();
    
    for i in 0..n-1 {
        total += distances[path[i]][path[i+1]];
    }
    
    // Add distance from last to first city to complete the cycle
    if n > 0 {
        total += distances[path[n-1]][path[0]];
    }
    
    total
}

/// More robust alternative to track entities with validation
#[derive(Resource, Default)]
pub struct ValidatedEntityTracker {
    /// Maps entity identifiers to the actual entities
    pub points_map: HashMap<usize, Entity>,
    pub paths_map: HashMap<(usize, usize), Entity>,
}
