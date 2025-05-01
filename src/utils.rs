use bevy::prelude::*;
use std::collections::HashMap;

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

/// More robust alternative to track entities with validation
#[derive(Resource, Default)]
pub struct ValidatedEntityTracker {
    /// Maps entity identifiers to the actual entities
    pub points_map: HashMap<usize, Entity>,
    pub paths_map: HashMap<(usize, usize), Entity>,
}

pub fn calculate_distance(path: &[usize], distances: &[Vec<f32>]) -> f32 {
    let mut total_distance = 0.0;
    let len = path.len();

    for i in 0..len - 1 {
        total_distance += distances[path[i]][path[i + 1]];
    }

    total_distance += distances[path[len - 1]][path[0]];
    total_distance
}

// Add this helper function for string formatting
pub fn format_with_args<T: std::fmt::Display>(format_str: &str, args: T) -> String {
    format!("{}", format!("{}", args).replace("{}", format_str))
}
