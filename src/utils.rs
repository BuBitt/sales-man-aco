use bevy::prelude::*;

pub fn safe_despawn_collection(
    commands: &mut Commands,
    entities: &mut Vec<Entity>,
) {
    let mut to_remove = Vec::new();
    for (i, &entity) in entities.iter().enumerate() {
        if commands.get_entity(entity).is_some() {
            commands.entity(entity).despawn_recursive();
            to_remove.push(i);
        }
    }

    to_remove.sort_by(|a, b| b.cmp(a));
    for index in to_remove {
        entities.remove(index);
    }
}

pub fn safe_despawn(commands: &mut Commands, entity: Entity) {
    if commands.get_entity(entity).is_some() {
        commands.entity(entity).despawn_recursive();
    }
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
