use crate::utils::calculate_distance;

pub fn apply_2opt(path: &mut Vec<usize>, distances: &[Vec<f32>]) -> f32 {
    let n = path.len();
    let mut improved = true;
    let mut distance = calculate_distance(path, distances);

    while improved {
        improved = false;

        for i in 0..n - 2 {
            for j in i + 2..n {
                let current_distance =
                    distances[path[i]][path[i + 1]] +
                    distances[path[j]][path[(j + 1) % n]];

                let new_distance =
                    distances[path[i]][path[j]] +
                    distances[path[i + 1]][path[(j + 1) % n]];

                if new_distance < current_distance {
                    path[i + 1..=j].reverse();
                    distance = calculate_distance(path, distances);
                    improved = true;
                    break;
                }
            }
            if improved {
                break;
            }
        }
    }

    distance
}
