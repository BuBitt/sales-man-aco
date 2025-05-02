//! # Constantes do Sistema
//!
//! Este módulo define valores constantes usados em toda a aplicação.
//! Centralizar estas constantes facilita ajustes e manutenção do código.

/// Número padrão de formigas usadas no algoritmo ACO
pub const DEFAULT_ANT_COUNT: usize = 20;

/// Número máximo de iterações permitidas pelo algoritmo
pub const MAX_ITERATIONS: u32 = 1000;

/// Importância do feromônio na escolha do caminho pela formiga (α)
pub const DEFAULT_ALPHA: f32 = 1.0;

/// Importância da distância na escolha do caminho pela formiga (β)
pub const DEFAULT_BETA: f32 = 2.0;

/// Taxa de evaporação do feromônio após cada iteração (ρ)
pub const DEFAULT_RHO: f32 = 0.5;

/// Fator de depósito de feromônio (Q)
/// Determina quanto feromônio é depositado em relação à qualidade da solução
pub const DEFAULT_Q: f32 = 100.0;

/// Tamanho das listas de candidatos
/// Cada formiga considera apenas os N vizinhos mais próximos para cada cidade
pub const CANDIDATE_LIST_SIZE: usize = 20;

/// Critério de interrupção por estagnação
/// O algoritmo para se não houver melhoria após este número de iterações
pub const MAX_ITERATIONS_WITHOUT_IMPROVEMENT: u32 = 50;

/// Limite para ativar processamento paralelo
/// Para problemas com mais pontos que este limite, usa-se paralelização
pub const PARALLEL_THRESHOLD: usize = 100;

// pub const LOCAL_SEARCH_FREQUENCY: u32 = 10;
// pub const USE_LOCAL_SEARCH: bool = true;
