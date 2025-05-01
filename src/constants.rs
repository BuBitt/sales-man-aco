pub const DEFAULT_ANT_COUNT: usize = 20;
pub const MAX_ITERATIONS: u32 = 1000;
pub const DEFAULT_ALPHA: f32 = 1.0; // Pheromone importance
pub const DEFAULT_BETA: f32 = 2.0;  // Distance importance
pub const DEFAULT_RHO: f32 = 0.5;   // Pheromone evaporation rate
pub const DEFAULT_Q: f32 = 100.0;   // Pheromone deposit factor
pub const CANDIDATE_LIST_SIZE: usize = 20; // Agora usado corretamente - aumentado para 20
pub const MAX_ITERATIONS_WITHOUT_IMPROVEMENT: u32 = 50; // Early stopping criterion

// Novas constantes para otimização
pub const PARALLEL_THRESHOLD: usize = 100; // Número de pontos acima do qual usar processamento paralelo
pub const LOCAL_SEARCH_FREQUENCY: u32 = 10; // Aplicar busca local a cada X iterações
