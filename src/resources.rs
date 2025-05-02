use bevy::prelude::*;
use crate::constants::*;
use std::time::{Duration, Instant};

#[derive(Default, Resource)]
pub struct EntityTracker {
    pub point_entities: Vec<Entity>,
    pub path_entities: Vec<Entity>,
}

#[derive(Resource)]
pub struct DistanceMatrix {
    pub distances: Vec<Vec<f32>>,
}

impl Default for DistanceMatrix {
    fn default() -> Self {
        Self {
            distances: Vec::new(),
        }
    }
}

#[derive(Resource)]
pub struct CandidateList {
    pub nearest_neighbors: Vec<Vec<usize>>,
}

impl Default for CandidateList {
    fn default() -> Self {
        Self {
            nearest_neighbors: Vec::new(),
        }
    }
}

#[derive(Default, Resource)]
pub struct AcoState {
    pub running: bool,
    pub start_time: Option<Instant>,
    pub elapsed_time: Duration,
    pub iterations: u32,
    pub pheromones: Vec<Vec<f32>>,
    pub iterations_since_improvement: u32,
}

#[derive(Default, Resource)]
pub struct Points {
    pub positions: Vec<Vec2>,
    pub count: usize,
}

impl Points {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            count: 20,
        }
    }
}

#[derive(Default, Resource)]
pub struct UiState {
    pub interacting_with_ui: bool,
}

#[derive(Resource)]
pub struct AcoParameters {
    pub ant_count: usize,
    pub alpha: f32,
    pub beta: f32,
    pub rho: f32,
    pub q: f32,
}

impl Default for AcoParameters {
    fn default() -> Self {
        Self {
            ant_count: DEFAULT_ANT_COUNT,
            alpha: DEFAULT_ALPHA,
            beta: DEFAULT_BETA,
            rho: DEFAULT_RHO,
            q: DEFAULT_Q,
        }
    }
}

#[derive(Default, Resource)]
pub struct BestPath {
    pub path: Vec<usize>,
    pub distance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Portuguese,
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

#[derive(Resource, Default)]
pub struct AppLanguage {
    pub current: Language,
}

/// Configuração para gerenciamento otimizado de memória
#[derive(Resource)]
pub struct MemoryConfig {
    pub use_arena_allocation: bool,
    pub reuse_vectors: bool,
    pub vector_pool_size: usize,
    pub max_points_in_view: usize,
}

/// Pool de vetores reutilizáveis para reduzir alocações
#[derive(Resource)]
pub struct VectorPool {
    pub float_vectors: Vec<Vec<f32>>,
    pub int_vectors: Vec<Vec<usize>>,
    pub bool_vectors: Vec<Vec<bool>>,
    pub available_floats: Vec<usize>,
    pub available_ints: Vec<usize>,
    pub available_bools: Vec<usize>,
}

impl VectorPool {
    pub fn new(initial_size: usize) -> Self {
        Self {
            float_vectors: Vec::with_capacity(initial_size),
            int_vectors: Vec::with_capacity(initial_size),
            bool_vectors: Vec::with_capacity(initial_size),
            available_floats: Vec::with_capacity(initial_size),
            available_ints: Vec::with_capacity(initial_size),
            available_bools: Vec::with_capacity(initial_size),
        }
    }
    
    pub fn get_float_vec(&mut self, size: usize) -> usize {
        if let Some(idx) = self.available_floats.pop() {
            let vec = &mut self.float_vectors[idx];
            vec.clear();
            vec.resize(size, 0.0);
            idx
        } else {
            let idx = self.float_vectors.len();
            self.float_vectors.push(vec![0.0; size]);
            idx
        }
    }
    
    // Métodos similares para int_vectors e bool_vectors
    // ...
    
    pub fn release_float_vec(&mut self, idx: usize) {
        self.available_floats.push(idx);
    }
    
    // Métodos similares para release de outros tipos...
}

/// Enumeração para versões OpenGL
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlVersion {
    GL4_3,
    GL4_6,
}

/// Configuração para aceleração em GPU
#[derive(Resource)]
pub struct GpuConfig {
    pub enabled: bool,
    pub use_gpu_threshold: usize,
    pub compute_shader_path: &'static str,
    pub opengl_version: GlVersion,
    pub synchronize_with_cpu: bool,
}

/// Estado de sincronização entre GPU e CPU
#[derive(Resource, Default)]
pub struct GpuSyncState {
    pub computation_completed: bool,
    pub iteration_ready: bool,
    pub last_processed_iteration: u32,
    pub frame_skip_counter: u32,
}

/// Recursos para cálculos em GPU
#[derive(Resource, Default)]
pub struct GpuResources {
    pub initialized: bool,
    pub shader_handle: Option<Handle<Shader>>,
    pub distance_buffer: Option<u32>,
    pub pheromone_buffer: Option<u32>,
    pub result_buffer: Option<u32>,
}
