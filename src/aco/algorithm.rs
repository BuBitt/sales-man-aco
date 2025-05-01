use rand::prelude::*;

pub fn calculate_ant_path(
    pheromones: &[Vec<f32>], 
    distances: &[Vec<f32>], 
    alpha: f32, 
    beta: f32, 
    start_city: usize,
    candidate_lists: Option<&[Vec<usize>]>
) -> (Vec<usize>, f32) {
    let n = distances.len();
    if n == 0 {
        return (Vec::new(), 0.0);
    }
    
    let mut path = Vec::with_capacity(n + 1); // +1 para incluir o retorno à cidade inicial
    path.push(start_city);
    
    let mut visited = vec![false; n];
    visited[start_city] = true;
    
    let mut distance = 0.0;
    let mut current_city = start_city;
    let mut remaining = n - 1;
    
    // Cache para cálculos repetidos
    let mut probs_cache = Vec::with_capacity(n);
    
    // Construção do caminho otimizada
    while remaining > 0 {
        let next_city = if let Some(candidates) = candidate_lists {
            // Primeiro tenta usar a lista de candidatos (mais eficiente)
            let candidate_cities = &candidates[current_city];
            let mut found_valid = false;
            let mut best_city = 0;
            let mut best_prob = 0.0;
            
            // Tenta encontrar próxima cidade na lista de candidatos
            for &candidate in candidate_cities {
                if !visited[candidate] {
                    let pheromone = pheromones[current_city][candidate].powf(alpha);
                    let heuristic = (1.0 / distances[current_city][candidate]).powf(beta);
                    let probability = pheromone * heuristic;
                    
                    if probability > best_prob {
                        best_prob = probability;
                        best_city = candidate;
                        found_valid = true;
                    }
                }
            }
            
            // Se encontrou candidato válido, usa ele
            if found_valid {
                best_city
            } else {
                // Caso contrário, faz seleção completa
                select_next_city(current_city, &visited, pheromones, distances, alpha, beta, &mut probs_cache)
            }
        } else {
            // Não tem lista de candidatos, faz busca completa
            select_next_city(current_city, &visited, pheromones, distances, alpha, beta, &mut probs_cache)
        };
        
        distance += distances[current_city][next_city];
        path.push(next_city);
        visited[next_city] = true;
        current_city = next_city;
        remaining -= 1;
    }
    
    // Fecha o circuito voltando à cidade inicial
    distance += distances[current_city][start_city];
    
    (path, distance)
}

// Função otimizada para seleção da próxima cidade
#[inline]
fn select_next_city(
    current_city: usize,
    visited: &[bool],
    pheromones: &[Vec<f32>],
    distances: &[Vec<f32>],
    alpha: f32,
    beta: f32,
    probs_cache: &mut Vec<(usize, f32)>
) -> usize {
    let n = distances.len();
    probs_cache.clear(); // Reutiliza o vetor para evitar realocações
    
    let mut total = 0.0;
    
    // Calcula probabilidades uma única vez - otimizado
    for i in 0..n {
        if !visited[i] {
            let pheromone = pheromones[current_city][i].powf(alpha);
            let heuristic = if distances[current_city][i] > 0.0 {
                (1.0 / distances[current_city][i]).powf(beta)
            } else {
                f32::MAX / 2.0 // Evita overflow
            };
            
            let prob = pheromone * heuristic;
            probs_cache.push((i, prob));
            total += prob;
        }
    }
    
    // Abordagem de seleção por roleta otimizada
    if total > 0.0 {
        let mut rand_val = thread_rng().gen::<f32>() * total;
        
        for &(city, prob) in probs_cache.iter() {
            rand_val -= prob;
            if rand_val <= 0.0 {
                return city;
            }
        }
    }
    
    // Caso de contingência - retorna a primeira cidade não visitada
    for i in 0..n {
        if !visited[i] {
            return i;
        }
    }
    
    // Não deve chegar aqui, mas por segurança
    current_city
}
