use rand::prelude::*;
use crate::resources::AcoParameters;

pub struct Ant {
    pub visited: Vec<bool>,
    pub path: Vec<usize>,
    pub distance: f32,
}

impl Ant {
    pub fn new(point_count: usize) -> Self {
        Self {
            visited: vec![false; point_count],
            path: Vec::with_capacity(point_count),
            distance: 0.0,
        }
    }

    pub fn construct_solution(
        &mut self,
        distances: &[Vec<f32>],
        pheromones: &[Vec<f32>],
        rng: &mut ThreadRng,
        params: &AcoParameters,
    ) {
        let point_count = distances.len();
        self.visited.fill(false);
        self.path.clear();
        self.distance = 0.0;

        let start = rng.gen_range(0..point_count);
        self.path.push(start);
        self.visited[start] = true;

        while self.path.len() < point_count {
            let current = *self.path.last().unwrap();
            let next = self.select_next_city(current, pheromones, rng, params, distances);

            let distance = distances[current][next];
            self.distance += distance;

            self.path.push(next);
            self.visited[next] = true;
        }

        let first = self.path[0];
        let last = self.path[point_count - 1];
        self.distance += distances[last][first];
    }

    pub fn select_next_city(
        &self,
        current: usize,
        pheromones: &[Vec<f32>],
        rng: &mut ThreadRng,
        params: &AcoParameters,
        distances: &[Vec<f32>],
    ) -> usize {
        let point_count = distances.len();

        let mut total_prob = 0.0;
        let mut probabilities = vec![0.0; point_count];

        for i in 0..point_count {
            if !self.visited[i] {
                let distance = distances[current][i];
                let pheromone = pheromones[current][i];

                let distance_factor = if distance < 0.0001 { 1000.0 } else { 1.0 / distance };

                probabilities[i] = pheromone.powf(params.alpha) * distance_factor.powf(params.beta);
                total_prob += probabilities[i];
            }
        }

        let mut choice = rng.gen::<f32>() * total_prob;
        for i in 0..point_count {
            if !self.visited[i] {
                choice -= probabilities[i];
                if choice <= 0.0 {
                    return i;
                }
            }
        }

        for i in 0..point_count {
            if !self.visited[i] {
                return i;
            }
        }

        unreachable!("Should have found an unvisited city")
    }

    pub fn select_next_city_with_candidates(
        &self,
        current: usize,
        pheromones: &[Vec<f32>],
        rng: &mut ThreadRng,
        params: &AcoParameters,
        distances: &[Vec<f32>],
        candidate_lists: &[Vec<usize>],
    ) -> usize {
        let candidates = &candidate_lists[current];
        let unvisited_candidates: Vec<usize> = candidates.iter()
            .filter(|&&city| !self.visited[city])
            .copied()
            .collect();

        if !unvisited_candidates.is_empty() {
            let mut total_prob = 0.0;
            let mut probabilities = vec![0.0; unvisited_candidates.len()];

            for (i, &city) in unvisited_candidates.iter().enumerate() {
                let distance = distances[current][city];
                let pheromone = pheromones[current][city];

                let distance_factor = if distance < 0.0001 { 1000.0 } else { 1.0 / distance };
                probabilities[i] = pheromone.powf(params.alpha) * distance_factor.powf(params.beta);
                total_prob += probabilities[i];
            }

            if total_prob > 0.0 {
                let mut choice = rng.gen::<f32>() * total_prob;
                for (i, &city) in unvisited_candidates.iter().enumerate() {
                    choice -= probabilities[i];
                    if choice <= 0.0 {
                        return city;
                    }
                }
            }
        }

        self.select_next_city(current, pheromones, rng, params, distances)
    }
}
