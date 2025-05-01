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
