mod algorithm;

use rand::prelude::*;

// Expanded Ant struct with all required fields and methods
pub struct Ant {
    pub path: Vec<usize>,
    pub visited: Vec<bool>,
    pub distance: f32,
    pub current_city: usize,
}

impl Ant {
    pub fn new(city_count: usize) -> Self {
        Self {
            path: Vec::with_capacity(city_count),
            visited: vec![false; city_count],
            distance: f32::MAX,
            current_city: 0,
        }
    }
    
    pub fn select_next_city_with_candidates(
        &mut self, 
        pheromones: &[Vec<f32>], 
        distances: &[Vec<f32>],
        candidates: &[Vec<usize>],
        alpha: f32,
        beta: f32,
        candidate_list_size: usize,
    ) -> usize {
        let current = self.current_city;
        
        // Safety check - make sure current city is valid index for candidates
        if current >= candidates.len() {
            // Fallback to standard selection if current city index is out of bounds
            return self.select_next_city(pheromones, distances, alpha, beta);
        }
        
        // Get the candidate list for the current city
        let candidate_list = &candidates[current];
        
        // Safety check: Use a safe candidate size that won't exceed bounds
        let safe_size = candidate_list_size.min(candidate_list.len());
        
        // Initialize variables for selection
        let mut total = 0.0;
        let mut probabilities = vec![];
        
        // Calculate probabilities only for candidates that are not visited
        // and respect the safe candidate list size
        for i in 0..safe_size {
            if i >= candidate_list.len() {
                break; // Additional safety check
            }
            
            let candidate = candidate_list[i];
            if candidate < self.visited.len() && !self.visited[candidate] {
                let pheromone = pheromones[current][candidate].powf(alpha);
                let distance = (1.0 / distances[current][candidate]).powf(beta);
                let prob = pheromone * distance;
                
                total += prob;
                probabilities.push((candidate, prob));
            }
        }
        
        // If no suitable candidates in the candidate list, check all unvisited cities
        if probabilities.is_empty() {
            return self.select_next_city(pheromones, distances, alpha, beta);
        }
        
        // Select city based on probabilities
        let random = rand::random::<f32>() * total;
        let mut cumulative = 0.0;
        
        // Fix: iterate over a reference to avoid moving the vector
        for (city, prob) in &probabilities {
            cumulative += *prob;
            if cumulative >= random {
                return *city;
            }
        }
        
        // Fallback: return the first city in our probability list
        if let Some((city, _)) = probabilities.first() {
            return *city;
        }
        
        // Final fallback - use standard selection method
        self.select_next_city(pheromones, distances, alpha, beta)
    }
    
    pub fn select_next_city(
        &mut self,
        pheromones: &[Vec<f32>], 
        distances: &[Vec<f32>],
        alpha: f32,
        beta: f32,
    ) -> usize {
        let current = self.current_city;
        
        // Initialize variables for selection
        let mut total = 0.0;
        let mut probabilities = vec![];
        
        // Calculate probabilities for all unvisited cities
        for city in 0..self.visited.len() {
            if !self.visited[city] {
                let pheromone = pheromones[current][city].powf(alpha);
                let distance = (1.0 / distances[current][city]).powf(beta);
                let prob = pheromone * distance;
                
                total += prob;
                probabilities.push((city, prob));
            }
        }
        
        // Select city based on probabilities
        let random = rand::random::<f32>() * total;
        let mut cumulative = 0.0;
        
        // Fix: iterate over a reference to avoid moving the vector
        for (city, prob) in &probabilities {
            cumulative += *prob;
            if cumulative >= random {
                return *city;
            }
        }
        
        // Fallback: If for some reason we didn't select a city, return the first unvisited city
        for city in 0..self.visited.len() {
            if !self.visited[city] {
                return city;
            }
        }
        
        // This should never happen if visited tracking is correct
        current // Return current city as a last resort (though this isn't a good choice)
    }
    
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
        self.current_city = start_city;
        
        // Start with initial city
        self.path.push(start_city);
        self.visited[start_city] = true;
        
        // Build path
        for _ in 1..n {
            let current = *self.path.last().unwrap();
            
            // Select next city using regular method (no candidates)
            let next = self.select_next_city(
                pheromones,
                distances,
                params.alpha,
                params.beta,
            );
            
            self.distance += distances[current][next];
            self.path.push(next);
            self.visited[next] = true;
            self.current_city = next;
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
