//! # Constantes do Sistema
//!
//! Este módulo define valores constantes usados em toda a aplicação.
//! Centralizar estas constantes facilita ajustes e manutenção do código.
//! 
//! Nota: A maioria das configurações foi movida para o arquivo config.toml

/// Limite para ativar processamento paralelo
/// Para problemas com mais pontos que este limite, usa-se paralelização
pub const PARALLEL_THRESHOLD: usize = 100;

/// Tamanho das listas de candidatos
/// Cada formiga considera apenas os N vizinhos mais próximos para cada cidade
pub const CANDIDATE_LIST_SIZE: usize = 20;

// Estas constantes agora são definidas no arquivo de configuração
// pub const DEFAULT_ANT_COUNT: usize = 20;
// pub const MAX_ITERATIONS: u32 = 1000;
// pub const DEFAULT_ALPHA: f32 = 1.0;
// pub const DEFAULT_BETA: f32 = 2.0;
// pub const DEFAULT_RHO: f32 = 0.5;
// pub const DEFAULT_Q: f32 = 100.0;
// pub const MAX_ITERATIONS_WITHOUT_IMPROVEMENT: u32 = 50;
