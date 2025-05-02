#version 460

layout(local_size_x = 16, local_size_y = 16) in;

// Buffer de entrada: posições dos pontos
layout(std430, binding = 0) buffer PointPositions {
    vec2 positions[];
};

// Buffer de saída: matriz de distâncias
layout(std430, binding = 1) buffer DistanceMatrix {
    float distances[];
};

// Buffer de saída: matriz de feromônios atualizada
layout(std430, binding = 2) buffer PheromoneMatrix {
    float pheromones[];
};

// Parâmetros uniformes
layout(std140, binding = 3) uniform Params {
    float alpha;
    float beta;
    float rho;
    int point_count;
};

// Calcula distâncias entre todos os pares de pontos
void main() {
    uint i = gl_GlobalInvocationID.x;
    uint j = gl_GlobalInvocationID.y;
    
    // Verifica se estamos dentro dos limites
    if (i >= point_count || j >= point_count) {
        return;
    }
    
    // Se for o mesmo ponto, distância = 0
    if (i == j) {
        distances[i * point_count + j] = 0.0;
        return;
    }
    
    // Calcula distância euclidiana
    vec2 pos_i = positions[i];
    vec2 pos_j = positions[j];
    vec2 diff = pos_j - pos_i;
    float dist = length(diff);
    
    // Armazena no buffer de saída
    distances[i * point_count + j] = dist;
}
