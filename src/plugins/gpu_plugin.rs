use bevy::{
    prelude::*,
    render::renderer::RenderDevice,
};
use crate::resources::*;

/// Plugin para acelerar cálculos usando a GPU
pub struct GpuAccelerationPlugin;

impl Plugin for GpuAccelerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gpu)
           .add_systems(
               Update, 
               update_gpu_resources
                   .run_if(|config: Res<GpuConfig>| config.enabled)
           );
    }
}

/// Configura recursos necessários para computação em GPU
fn setup_gpu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gpu_config: Res<GpuConfig>,
) {
    if !gpu_config.enabled {
        return;
    }
    
    info!("Inicializando recursos para aceleração em GPU (OpenGL {:?})", gpu_config.opengl_version);
    
    // Add underscore to mark variable as intentionally unused
    let _shader_handle: Handle<Shader> = asset_server.load(gpu_config.compute_shader_path);
    
    commands.insert_resource(GpuResources {
        initialized: false,
        shader_handle: None,
        distance_buffer: None,
        pheromone_buffer: None,
        result_buffer: None,
    });
}

/// Atualiza buffers e outros recursos da GPU quando necessário
fn update_gpu_resources(
    points: Res<Points>,
    gpu_config: Res<GpuConfig>,
    mut gpu_resources: ResMut<GpuResources>,
    _render_device: Res<RenderDevice>,
) {
    // Só executar computações em GPU se exceder o limite configurado
    if points.positions.len() <= gpu_config.use_gpu_threshold {
        return;
    }
    
    // Inicializa buffers OpenGL apenas na primeira vez ou quando redimensionar
    if !gpu_resources.initialized || gpu_resources.distance_buffer.is_none() {
        info!("Criando buffers GPU para {} pontos", points.positions.len());
        
        // Simulação básica de criação de buffers
        gpu_resources.initialized = true;
        gpu_resources.distance_buffer = Some(1); // Valor simulado
        gpu_resources.pheromone_buffer = Some(2);
        gpu_resources.result_buffer = Some(3);
    }
}
