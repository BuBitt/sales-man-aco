// TSP ACO Compute Shader for distance calculation

@group(0) @binding(0)
var<storage, read> positions: array<vec2<f32>>;

@group(0) @binding(1)
var<storage, read_write> distances: array<f32>;

@group(0) @binding(2)
var<storage, read_write> pheromones: array<f32>;

struct Params {
    alpha: f32,
    beta: f32,
    rho: f32,
    point_count: u32,
};

@group(0) @binding(3)
var<uniform> params: Params;

// Define compute workgroup size
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    let j = global_id.y;
    
    // Boundary check
    if (i >= params.point_count || j >= params.point_count) {
        return;
    }
    
    // Calculate linear index for 2D matrix stored as 1D array
    let index = i * params.point_count + j;
    
    // Self distance is always 0
    if (i == j) {
        distances[index] = 0.0;
        return;
    }
    
    // Calculate Euclidean distance
    let pos_i = positions[i];
    let pos_j = positions[j];
    let diff = pos_j - pos_i;
    let dist = length(diff);
    
    // Store the result
    distances[index] = dist;
}
