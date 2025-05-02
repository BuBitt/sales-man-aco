use bevy::prelude::*;
use rand::prelude::*;
use rayon::prelude::*;
use std::time::Instant;
use crate::resources::*;
use crate::aco::{Ant, apply_2opt};

/// Sistema principal que executa o algoritmo ACO
/// 
/// Este sistema é responsável por executar o algoritmo de otimização
/// de colônia de formigas para resolver o TSP.
pub fn run_aco_algorithm(
    mut aco_state: ResMut<AcoState>,
    points: Res<Points>,
    mut best_path: ResMut<BestPath>,
    aco_params: Res<AcoParameters>,
    distance_matrix: Res<DistanceMatrix>,
    candidate_lists: Res<CandidateList>,
    gpu_config: Res<GpuConfig>,
    mut gpu_sync: ResMut<GpuSyncState>,
) {
    if !aco_state.running || points.positions.is_empty() {
        return;
    }

    // Atualiza estado de sincronização GPU para esta iteração
    if points.positions.len() > gpu_config.use_gpu_threshold && gpu_config.enabled {
        gpu_sync.iteration_ready = false;
    }

    // Iniciar cronômetro se ainda não iniciado
    if aco_state.start_time.is_none() {
        aco_state.start_time = Some(Instant::now());
    }

    // Use configurable values from AcoState if available, otherwise use constants
    const DEFAULT_MAX_ITERATIONS: u32 = 1000;
    const DEFAULT_MAX_ITERATIONS_WITHOUT_IMPROVEMENT: u32 = 50;

    let max_iterations = aco_state.max_iterations.unwrap_or(DEFAULT_MAX_ITERATIONS);
    let max_iterations_no_improvement = aco_state.max_iterations_no_improvement.unwrap_or(DEFAULT_MAX_ITERATIONS_WITHOUT_IMPROVEMENT);

    // Se já atingiu o máximo de iterações, para o algoritmo
    if aco_state.iterations >= max_iterations {
        aco_state.running = false;
        if let Some(start_time) = aco_state.start_time {
            aco_state.elapsed_time += Instant::now().duration_since(start_time);
            aco_state.start_time = None;
        }
        return;
    }

    // Se não houve melhoria após várias iterações, para o algoritmo
    if aco_state.iterations_since_improvement >= max_iterations_no_improvement {
        aco_state.running = false;
        if let Some(start_time) = aco_state.start_time {
            aco_state.elapsed_time += Instant::now().duration_since(start_time);
            aco_state.start_time = None;
        }
        return;
    }

    let n = points.positions.len();
    
    // Em problemas grandes, usar um número reduzido mas suficiente de formigas
    let effective_ant_count = if n > crate::constants::PARALLEL_THRESHOLD {
        (aco_params.ant_count / 4).max(20) // Reduz número de formigas mas mantém mínimo de 20
    } else {
        aco_params.ant_count
    };

    // Incrementa iteração após todo o processamento
    aco_state.iterations += 1;
    aco_state.iterations_since_improvement += 1;
    
    // Criar formigas com o tamanho apropriado
    let mut ants: Vec<Ant> = (0..effective_ant_count).map(|_| Ant::new(n)).collect();

    // Verificação de segurança para garantir acesso seguro à matriz de distâncias
    if distance_matrix.distances.len() != n || n == 0 {
        return;
    }

    // Distribuir formigas nos pontos de partida
    let mut starting_points: Vec<usize> = (0..n).collect();
    let mut rng = thread_rng();
    starting_points.shuffle(&mut rng);

    // Get the candidate list size from aco_state or use default
    let candidate_list_size = aco_state.candidate_list_size.unwrap_or(20);

    ants.par_iter_mut().for_each(|ant| {
        let mut local_rng = thread_rng();
        let start = if !starting_points.is_empty() {
            starting_points[local_rng.gen_range(0..starting_points.len())]
        } else {
            local_rng.gen_range(0..n)
        };
        
        ant.visited.fill(false);
        ant.path.clear();
        ant.distance = 0.0;
        
        ant.path.push(start);
        ant.visited[start] = true;
        
        while ant.path.len() < n {
            let current = *ant.path.last().unwrap();
            let next = if !candidate_lists.nearest_neighbors.is_empty() {
                ant.select_next_city_with_candidates(
                    &aco_state.pheromones,       // pheromones
                    &distance_matrix.distances,  // distances
                    &candidate_lists.nearest_neighbors, // candidates
                    aco_params.alpha,            // alpha
                    aco_params.beta,             // beta
                    candidate_list_size          // candidate_list_size
                )
            } else {
                ant.select_next_city(
                    &aco_state.pheromones,
                    &distance_matrix.distances,
                    aco_params.alpha,
                    aco_params.beta
                )
            };
            
            ant.distance += distance_matrix.distances[current][next];
            ant.path.push(next);
            ant.visited[next] = true;
        }
        
        let first = ant.path[0];
        let last = ant.path[n - 1];
        ant.distance += distance_matrix.distances[last][first];
    });

    let mut iteration_best_ant = &ants[0];
    for ant in &ants {
        if ant.distance < iteration_best_ant.distance {
            iteration_best_ant = ant;
        }
    }

    let mut improved_path = iteration_best_ant.path.clone();
    let improved_distance = apply_2opt(&mut improved_path, &distance_matrix.distances);
    
    if improved_distance < best_path.distance {
        best_path.path = improved_path;
        best_path.distance = improved_distance;
        aco_state.iterations_since_improvement = 0;
    }

    // Update pheromones
    for i in 0..n {
        for j in 0..n {
            aco_state.pheromones[i][j] *= 1.0 - aco_params.rho;
        }
    }

    // Add pheromones from ants
    for ant in &ants {
        let pheromone_amount = aco_params.q / ant.distance;
        for i in 0..n - 1 {
            let from = ant.path[i];
            let to = ant.path[i + 1];
            aco_state.pheromones[from][to] += pheromone_amount;
            aco_state.pheromones[to][from] += pheromone_amount;
        }

        let from = ant.path[n - 1];
        let to = ant.path[0];
        aco_state.pheromones[from][to] += pheromone_amount;
        aco_state.pheromones[to][from] += pheromone_amount;
    }

    // Add extra pheromones to best path
    let best_pheromone = 2.0 * aco_params.q / best_path.distance;
    for i in 0..best_path.path.len() - 1 {
        let from = best_path.path[i];
        let to = best_path.path[i + 1];
        aco_state.pheromones[from][to] += best_pheromone;
        aco_state.pheromones[to][from] += best_pheromone;
    }

    let from = best_path.path[n - 1];
    let to = best_path.path[0];
    aco_state.pheromones[from][to] += best_pheromone;
    aco_state.pheromones[to][from] += best_pheromone;
}
