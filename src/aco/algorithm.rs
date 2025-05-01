use rand::prelude::*;

pub fn calculate_ant_path(pheromones: &[Vec<f32>], distances: &[Vec<f32>], alpha: f32, beta: f32, start_city: usize) -> (Vec<usize>, f32) {
    let n = distances.len();
    let mut path = vec![start_city];
    let mut visited = vec![false; n];
    visited[start_city] = true;
    
    // Initialize distance to 0
    let mut distance = 0.0;
    
    // Build the path
    for _ in 1..n {
        let current_city = *path.last().unwrap();
        let next_city = select_next_city(current_city, &visited, pheromones, distances, alpha, beta);
        
        // Add to total distance using the distance matrix directly
        distance += distances[current_city][next_city];
        
        path.push(next_city);
        visited[next_city] = true;
    }
    
    // Add distance back to starting city to complete the tour
    if path.len() > 1 {
        let last = *path.last().unwrap();
        distance += distances[last][start_city];
    }
    
    (path, distance)
}

fn select_next_city(current_city: usize, visited: &[bool], pheromones: &[Vec<f32>], 
                   distances: &[Vec<f32>], alpha: f32, beta: f32) -> usize {
    let n = distances.len();
    
    // Calculate probabilities for each unvisited city
    let mut probabilities = vec![0.0; n];
    let mut total = 0.0;
    
    for i in 0..n {
        if !visited[i] && i != current_city {
            let pheromone = pheromones[current_city][i].powf(alpha);
            let heuristic = if distances[current_city][i] > 0.0 {
                (1.0 / distances[current_city][i]).powf(beta) 
            } else {
                f32::MAX
            };
            
            probabilities[i] = pheromone * heuristic;
            total += probabilities[i];
        }
    }
    
    // If all cities have been visited or total is zero, return the first unvisited city
    if total <= 0.0 {
        for i in 0..n {
            if !visited[i] {
                return i;
            }
        }
        return current_city; // Fallback to current city if all are visited
    }
    
    // Select city based on probability
    let mut rand_val = thread_rng().gen::<f32>() * total;
    
    for i in 0..n {
        if !visited[i] && probabilities[i] > 0.0 {
            rand_val -= probabilities[i];
            if rand_val <= 0.0 {
                return i;
            }
        }
    }
    
    // Fallback to first unvisited city
    for i in 0..n {
        if !visited[i] {
            return i;
        }
    }
    
    current_city  // This shouldn't happen, but as a fallback
}
