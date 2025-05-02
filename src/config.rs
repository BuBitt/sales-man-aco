//! Módulo para carregamento e gerenciamento de configuração
//! 
//! Este módulo fornece funcionalidade para carregar configurações
//! a partir de um arquivo TOML externo.

use bevy::prelude::*;
use bevy::window::WindowResolution;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::resources::*;

#[derive(Deserialize, Serialize, Debug, Resource)]
pub struct Config {
    pub app: AppConfig,
    pub rendering: RenderingConfig,
    pub memory: MemoryConfigFile,
    pub gpu: GpuConfigFile,
    pub aco: AcoConfigFile,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AppConfig {
    pub title: String,
    pub window_width: f32,
    pub window_height: f32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RenderingConfig {
    pub clear_color_r: f32,
    pub clear_color_g: f32, 
    pub clear_color_b: f32,
    pub clear_color_a: f32,
    pub watch_for_changes: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MemoryConfigFile {
    pub max_points_in_view: usize,
    pub use_arena_allocation: bool,
    pub reuse_vectors: bool,
    pub vector_pool_size: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GpuConfigFile {
    pub enabled: bool,
    pub use_gpu_threshold: usize,
    pub compute_shader_path: String,
    pub opengl_version: String,
    pub synchronize_with_cpu: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AcoConfigFile {
    pub ant_count: usize,
    pub alpha: f32,
    pub beta: f32,
    pub rho: f32,
    pub q: f32,
    pub candidate_list_size: usize,
    pub max_iterations: u32,
    pub max_iterations_without_improvement: u32,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Path::new("config.toml");
        
        // Tenta carregar o arquivo de configuração
        match fs::read_to_string(config_path) {
            Ok(content) => {
                match toml::from_str(&content) {
                    Ok(config) => {
                        info!("Configuração carregada com sucesso");
                        config
                    },
                    Err(e) => {
                        error!("Erro ao analisar o arquivo de configuração: {}", e);
                        Self::default()
                    }
                }
            },
            Err(e) => {
                warn!("Não foi possível ler o arquivo de configuração: {}", e);
                warn!("Usando configurações padrão");
                
                // Cria um arquivo de configuração padrão para uso futuro
                let default_config = Self::default();
                if let Ok(toml_string) = toml::to_string_pretty(&default_config) {
                    let _ = fs::write(config_path, toml_string);
                }
                
                default_config
            }
        }
    }
    
    // Converte o GlVersion de string para enum
    pub fn get_gl_version(&self) -> GlVersion {
        match self.gpu.opengl_version.as_str() {
            "GL4_3" => GlVersion::GL4_3,
            _ => GlVersion::GL4_6,
        }
    }
    
    // Cria a configuração de janela a partir do arquivo
    pub fn get_window(&self) -> Window {
        Window {
            title: self.app.title.clone(),
            resolution: WindowResolution::new(
                self.app.window_width,
                self.app.window_height
            ),
            ..default()
        }
    }
    
    // Cria a cor de limpeza a partir do arquivo
    pub fn get_clear_color(&self) -> ClearColor {
        ClearColor(Color::srgba(
            self.rendering.clear_color_r,
            self.rendering.clear_color_g,
            self.rendering.clear_color_b,
            self.rendering.clear_color_a
        ))
    }
    
    // Cria a configuração de memória
    pub fn get_memory_config(&self) -> MemoryConfig {
        MemoryConfig {
            max_points_in_view: self.memory.max_points_in_view,
            use_arena_allocation: self.memory.use_arena_allocation,
            reuse_vectors: self.memory.reuse_vectors,
            vector_pool_size: self.memory.vector_pool_size,
        }
    }
    
    // Cria a configuração de GPU
    pub fn get_gpu_config(&self) -> GpuConfig {
        GpuConfig {
            enabled: self.gpu.enabled,
            use_gpu_threshold: self.gpu.use_gpu_threshold,
            compute_shader_path: Box::leak(self.gpu.compute_shader_path.clone().into_boxed_str()),
            opengl_version: self.get_gl_version(),
            synchronize_with_cpu: self.gpu.synchronize_with_cpu,
        }
    }
    
    // Cria os parâmetros do ACO
    pub fn get_aco_parameters(&self) -> AcoParameters {
        AcoParameters {
            ant_count: self.aco.ant_count,
            alpha: self.aco.alpha,
            beta: self.aco.beta,
            rho: self.aco.rho,
            q: self.aco.q,
        }
    }
}

// Implementação padrão para quando o arquivo não existir
impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig {
                title: "TSP - Ant Colony Optimization".to_string(),
                window_width: 1280.0,
                window_height: 720.0,
            },
            rendering: RenderingConfig {
                clear_color_r: 0.1,
                clear_color_g: 0.1,
                clear_color_b: 0.1,
                clear_color_a: 1.0,
                watch_for_changes: false,
            },
            memory: MemoryConfigFile {
                max_points_in_view: 2000,
                use_arena_allocation: true,
                reuse_vectors: true,
                vector_pool_size: 512,
            },
            gpu: GpuConfigFile {
                enabled: true,
                use_gpu_threshold: 100,
                compute_shader_path: "shaders/aco_compute.wgsl".to_string(),
                opengl_version: "GL4_6".to_string(),
                synchronize_with_cpu: true,
            },
            aco: AcoConfigFile {
                ant_count: 20,
                alpha: 1.0,
                beta: 2.0,
                rho: 0.5,
                q: 100.0,
                candidate_list_size: 20,
                max_iterations: 1000,
                max_iterations_without_improvement: 50,
            },
        }
    }
}
