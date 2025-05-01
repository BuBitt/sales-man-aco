mod algorithm;

use rand::prelude::*;

// Expanded Ant struct with all required fields and methods
pub struct Ant {
    pub path: Vec<usize>,
    pub visited: Vec<bool>,
    pub distance: f32,
}

impl Ant {
    pub fn new(city_count: usize) -> Self {
        Self {
            path: Vec::with_capacity(city_count),
            visited: vec![false; city_count],
            distance: f32::MAX,
        }
    }
    
    // Fix method signature to match exactly how it's called in systems/algorithm.rs
    pub fn select_next_city_with_candidates(
        &mut self,
        current_city: usize,
        pheromones: &[Vec<f32>],
        rng: &mut ThreadRng,
        params: &bevy::prelude::Res<crate::resources::AcoParameters>,
        distances: &[Vec<f32>],
        candidate_list: &[Vec<usize>],
    ) -> usize {
        let alpha = params.alpha;
        let beta = params.beta;
        let n = distances.len();
        
        // Check candidates first for efficiency
        let candidates = &candidate_list[current_city];
        let mut valid_candidates = Vec::new();
        
        for &candidate in candidates {
            if !self.visited[candidate] {
                valid_candidates.push(candidate);
            }
        }
        
        // If we have valid candidates, select from them
        if !valid_candidates.is_empty() {
            // Calculate probabilities for valid candidates
            let mut probabilities = vec![0.0; n];
            let mut total = 0.0;
            
            for &i in &valid_candidates {
                let pheromone = pheromones[current_city][i].powf(alpha);
                let heuristic = if distances[current_city][i] > 0.0 {
                    (1.0 / distances[current_city][i]).powf(beta)
                } else {
                    f32::MAX
                };
                
                probabilities[i] = pheromone * heuristic;
                total += probabilities[i];
            }
            
            // If total probability is valid, select based on probability
            if total > 0.0 {
                let mut rand_val = rng.gen::<f32>() * total;
                
                for &i in &valid_candidates {
                    rand_val -= probabilities[i];
                    if rand_val <= 0.0 {
                        return i;
                    }
                }
            }
            
            // Fallback to first valid candidate
            return valid_candidates[0];
        }
        
        // If no valid candidates, fall back to regular selection
        self.select_next_city(current_city, pheromones, rng, params, distances)
    }
    
    // Fix method signature to match exactly how it's called in systems/algorithm.rs
    pub fn select_next_city(
        &mut self,
        current_city: usize,
        pheromones: &[Vec<f32>],
        rng: &mut ThreadRng,
        params: &bevy::prelude::Res<crate::resources::AcoParameters>,
        distances: &[Vec<f32>],
    ) -> usize {
        let alpha = params.alpha;
        let beta = params.beta;
        let n = distances.len();
        
        // Calculate probabilities for all unvisited cities
        let mut probabilities = vec![0.0; n];
        let mut total = 0.0;
        
        for i in 0..n {
            if !self.visited[i] {
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
        
        // If all cities visited or total probability is zero, return first unvisited
        if total <= 0.0 {
            for i in 0..n {
                if !self.visited[i] {
                    return i;
                }
            }
            return current_city; // Fallback if all are visited
        }
        
        // Select based on probability
        let mut rand_val = rng.gen::<f32>() * total;
        
        for i in 0..n {
            if !self.visited[i] && probabilities[i] > 0.0 {
                rand_val -= probabilities[i];
                if rand_val <= 0.0 {
                    return i;
                }
            }
        }
        
        // Fallback to first unvisited city
        for i in 0..n {
            if !self.visited[i] {
                return i;
            }
        }
        
        current_city // This shouldn't happen, but as a fallback
    }
    
    // Fix construct_solution to match parameters actually used
    pub fn construct_solution(
        &mut self,
        distances: &[Vec<f32>],
        pheromones: &[Vec<f32>],
        rng: &mut ThreadRng,
        params: &bevy::prelude::Res<crate::resources::AcoParameters>,
    ) {
        let n = distances.len();
        
        // Determine starting city randomly
        let start_city = rng.gen_range(0..n);
        
        // Reset state
        self.path.clear();
        self.visited.fill(false);
        self.distance = 0.0;
        
        // Start with initial city
        self.path.push(start_city);
        self.visited[start_city] = true;
        
        // Build path
        for _ in 1..n {
            let current = *self.path.last().unwrap();
            
            // Select next city using regular method (no candidates)
            let next = self.select_next_city(current, pheromones, rng, params, distances);
            
            self.distance += distances[current][next];
            self.path.push(next);
            self.visited[next] = true;
        }
        
        // Complete the tour by returning to the start
        if self.path.len() > 1 {
            let last = *self.path.last().unwrap();
            let first = self.path[0];
            self.distance += distances[last][first];
        }
    }
}

// Implement 2-opt optimization for TSP paths
pub fn apply_2opt(path: &mut Vec<usize>, distances: &[Vec<f32>]) -> f32 {
    let n = path.len();
    let mut improved = true;
    let mut current_distance = calculate_path_distance(path, distances);
    
    // Continue until no improvement is found
    while improved {
        improved = false;
        
        for i in 0..n-2 {
            for j in i+2..n {
                // Calculate the distance difference if we swap edges (i,i+1) and (j,j+1)
                // where j+1 wraps around to 0 if j is the last city
                let next_j = (j + 1) % n;
                
                // Current edges: (i -> i+1) + (j -> next_j)
                let current_edges = distances[path[i]][path[i+1]] + distances[path[j]][path[next_j]];
                
                // New edges after swap: (i -> j) + (i+1 -> next_j)
                let new_edges = distances[path[i]][path[j]] + distances[path[i+1]][path[next_j]];
                
                // If the new path would be shorter
                if new_edges < current_edges {
                    // Reverse the segment between i+1 and j
                    path[i+1..=j].reverse();
                    
                    // Recalculate the total distance
                    current_distance = calculate_path_distance(path, distances);
                    improved = true;
                }
            }
        }
    }
    
    current_distance
}

// Helper function to calculate the total distance of a path
fn calculate_path_distance(path: &[usize], distances: &[Vec<f32>]) -> f32 {
    let n = path.len();
    let mut total = 0.0;
    
    for i in 0..n-1 {
        total += distances[path[i]][path[i+1]];
    }
    
    // Add distance from last to first city
    if n > 0 {
        total += distances[path[n-1]][path[0]];
    }
    
    total
}
