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
    pub max_points_in_view: usize,
    pub use_arena_allocation: bool,    // Public without underscore
    pub reuse_vectors: bool,           // Public without underscore
    pub vector_pool_size: usize,       // Public without underscore
}

/// Pool simplificada de vetores reutilizáveis
#[derive(Resource)]
pub struct VectorPool {
    pub initialized: bool,
}

impl VectorPool {
    pub fn new(_size: usize) -> Self {
        Self {
            initialized: true,
        }
    }
}

/// Enumeração para versões OpenGL
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlVersion {
    GL4_3,   // Removed underscore
    GL4_6,
}

/// Configuração para aceleração em GPU
#[derive(Resource)]
pub struct GpuConfig {
    pub enabled: bool,
    pub use_gpu_threshold: usize,
    pub compute_shader_path: &'static str,
    pub opengl_version: GlVersion,
    pub synchronize_with_cpu: bool,  // Public without underscore
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
    pub shader_handle: Option<Handle<Shader>>, // Public without underscore
    pub distance_buffer: Option<u32>,
    pub pheromone_buffer: Option<u32>,
    pub result_buffer: Option<u32>,
}
