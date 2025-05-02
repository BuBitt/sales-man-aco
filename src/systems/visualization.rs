//! # Módulo de Visualização
//!
//! Este módulo contém sistemas para renderizar e atualizar a visualização gráfica
//! dos pontos, caminhos e outros elementos visuais da simulação.

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use crate::components::*;
use crate::resources::*;
use crate::utils::{safe_despawn, safe_despawn_collection};
use rayon::prelude::*;

/// Atualiza a visualização de pontos na tela
///
/// Este sistema cria ou atualiza as entidades visuais para representar cada ponto.
/// Ele é ativado quando a coleção de pontos é modificada.
///
/// # Argumentos
/// * `commands` - Para criar novas entidades
/// * `points` - A coleção de pontos a serem visualizados
/// * `entity_tracker` - Rastreia as entidades criadas para gerenciamento posterior
/// * `point_markers` - Query para obter entidades de ponto existentes
/// * `meshes` - Assets para criar formas visuais
/// * `materials` - Assets para definir aparência visual
pub fn update_points_visualization(
    mut commands: Commands,
    points: Res<Points>,
    mut entity_tracker: ResMut<EntityTracker>,
    point_markers: Query<Entity, With<PointMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Só atualiza quando os pontos mudam e não está vazio, ou quando a visualização precisa ser limpa
    if (!points.is_changed() && !point_markers.is_empty() && !entity_tracker.point_entities.is_empty()) 
        || points.positions.is_empty() {
        return;
    }

    // Remove pontos existentes
    for entity in point_markers.iter() {
        safe_despawn(&mut commands, entity);
    }

    entity_tracker.point_entities.clear();

    // Cria pontos visualmente atraentes com efeito de brilho
    let point_size = 7.0; // Tamanho aumentado para melhor visibilidade
    let circle = meshes.add(Circle::new(point_size));
    let point_material = materials.add(ColorMaterial::from(Color::srgba(0.9, 0.9, 1.0, 1.0))); // Branco azulado mais suave
    
    // Cria contorno para destaque
    let outline_size = point_size + 3.0;
    let outline = meshes.add(Circle::new(outline_size));
    let outline_material = materials.add(ColorMaterial::from(Color::srgba(0.4, 0.7, 1.0, 0.5))); // Azul com transparência

    for position in &points.positions {
        // Primeiro cria o contorno/glow
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: outline.clone().into(),
                material: outline_material.clone(),
                transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
                ..default()
            },
            PointMarker,
        ));
        
        // Depois cria o ponto principal
        let entity = commands.spawn((
            MaterialMesh2dBundle {
                mesh: circle.clone().into(),
                material: point_material.clone(),
                transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.1)), // Ligeiramente acima do contorno
                ..default()
            },
            PointMarker,
        )).id();
        
        entity_tracker.point_entities.push(entity);
    }
}

/// Atualiza a visualização do melhor caminho encontrado
///
/// Renderiza linhas entre os pontos que formam o melhor caminho atual,
/// permitindo visualizar a solução do TSP.
///
/// # Argumentos
/// * `commands` - Para criar novas entidades
/// * `best_path` - Contém a informação do melhor caminho encontrado
/// * `points` - Posições dos pontos no espaço 2D
/// * `entity_tracker` - Rastreia entidades para gerenciamento
/// * `best_path_lines` - Query para obter linhas existentes do melhor caminho
/// * `materials` - Assets para definir aparência visual
pub fn update_path_visualization(
    mut commands: Commands,
    best_path: Res<BestPath>,
    points: Res<Points>,
    mut entity_tracker: ResMut<EntityTracker>,
    best_path_lines: Query<Entity, With<BestPathLine>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Verifica se há um novo melhor caminho para visualizar
    if !best_path.is_changed() || best_path.path.is_empty() {
        return;
    }

    // Remove linhas existentes
    safe_despawn_collection(&mut commands, &mut entity_tracker.path_entities);

    for entity in best_path_lines.iter() {
        safe_despawn(&mut commands, entity);
    }

    let n = best_path.path.len();
    
    // Definir materiais para caminhos normais e destacados
    let _line_material = materials.add(ColorMaterial::from(Color::srgba(0.0, 0.9, 0.4, 0.7))); // Verde mais vibrante com transparência
    let _highlight_material = materials.add(ColorMaterial::from(Color::srgba(0.1, 1.0, 0.6, 0.9))); // Verde mais brilhante para destaques

    entity_tracker.path_entities.clear();
    for i in 0..n {
        let from_idx = best_path.path[i];
        let to_idx = best_path.path[(i + 1) % n];

        // Verifica índices fora dos limites para segurança
        if from_idx >= points.positions.len() || to_idx >= points.positions.len() {
            continue;
        }

        let from = points.positions[from_idx];
        let to = points.positions[to_idx];
        let direction = to - from;
        let length = direction.length();
        
        // Largura da linha variável baseada na distância (linhas mais longas são mais finas)
        let line_width = if length > 300.0 {
            2.0  // Linha mais fina para caminhos longos
        } else if length > 150.0 {
            2.5  // Largura média para caminhos médios
        } else {
            3.0  // Linhas mais grossas para caminhos curtos
        };

        // Primeiro, criar uma linha de "glow" mais larga
        let glow_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(0.0, 0.8, 0.3, 0.3),
                    custom_size: Some(Vec2::new(length, line_width + 3.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0, 0.0),
                    rotation: Quat::from_rotation_z(direction.y.atan2(direction.x)),
                    ..default()
                },
                ..default()
            },
            BestPathLine,
        )).id();
        entity_tracker.path_entities.push(glow_entity);
        
        // Depois, criar a linha principal
        // Escolhe a cor baseada na posição no caminho (destaca as primeiras conexões)
        let line_color = if i < n / 5 {
            Color::srgba(0.1, 1.0, 0.6, 0.9) // Cor destacada para primeiras conexões
        } else {
            Color::srgba(0.0, 0.9, 0.4, 0.7) // Cor padrão
        };
        
        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: line_color, // Usar a cor diretamente em vez de tentar extraí-la do material
                    custom_size: Some(Vec2::new(length, line_width)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0, 0.1),
                    rotation: Quat::from_rotation_z(direction.y.atan2(direction.x)),
                    ..default()
                },
                ..default()
            },
            BestPathLine,
        )).id();
        entity_tracker.path_entities.push(entity);
    }
}

/// Atualiza a matriz de distâncias entre todos os pares de pontos
///
/// Calcula a distância euclidiana entre cada par de pontos e
/// armazena os resultados em uma matriz para uso eficiente pelo algoritmo ACO.
///
/// # Argumentos
/// * `points` - As posições de todos os pontos no espaço 2D
/// * `distance_matrix` - A matriz de distâncias a ser atualizada
pub fn update_distance_matrix(
    points: Res<Points>,
    mut distance_matrix: ResMut<DistanceMatrix>,
) {
    // Só recalcula quando os pontos mudam e existem pontos
    if points.is_changed() && !points.positions.is_empty() {
        let n = points.positions.len();
        let mut distances = vec![vec![0.0; n]; n];
        
        // Para conjuntos grandes de pontos, usa paralelismo para calcular distâncias
        if n > crate::constants::PARALLEL_THRESHOLD {
            // Dividir o trabalho em chunks por linhas da matriz
            let chunk_size = n.max(1) / num_cpus::get().max(1);
            let _chunk_size = chunk_size.max(1); // Garantir tamanho mínimo de 1
            
            // Processamento paralelo por linha da matriz
            let results: Vec<(usize, Vec<f32>)> = (0..n).into_par_iter().map(|i| {
                let mut row = vec![0.0; n];
                for j in 0..n {
                    if i != j {
                        row[j] = points.positions[i].distance(points.positions[j]);
                    }
                }
                (i, row)
            }).collect();
            
            // Integrar os resultados computados em paralelo
            for (i, row) in results {
                distances[i] = row;
            }
        } else {
            // Implementação sequencial para conjuntos pequenos
            for i in 0..n {
                for j in 0..n {
                    if i != j {
                        let dist = points.positions[i].distance(points.positions[j]);
                        distances[i][j] = dist;
                    }
                }
            }
        }
        
        distance_matrix.distances = distances;
        distance_matrix.set_changed();
    }
}

/// Atualiza as listas de candidatos para cada ponto
///
/// Para cada ponto, determina os `k` vizinhos mais próximos para uso
/// no algoritmo ACO. Isso reduz significativamente o espaço de busca.
///
/// # Argumentos
/// * `points` - As posições de todos os pontos no espaço 2D
/// * `distances` - A matriz de distâncias entre os pontos
/// * `candidate_lists` - As listas de candidatos a serem atualizadas
pub fn update_candidate_lists(
    points: Res<Points>,
    distances: Res<DistanceMatrix>,
    mut candidate_lists: ResMut<CandidateList>,
) {
    let n = points.positions.len();
    
    // Se não houver pontos, limpa a lista de candidatos e retorna
    if n == 0 {
        candidate_lists.nearest_neighbors = Vec::new();
        return;
    }
    
    // Verificações de segurança para a matriz de distâncias
    if distances.distances.is_empty() {
        candidate_lists.nearest_neighbors = Vec::new();
        return;
    }
    
    if distances.distances.len() != n {
        candidate_lists.nearest_neighbors = Vec::new();
        return;
    }
    
    for row in &distances.distances {
        if row.len() != n {
            candidate_lists.nearest_neighbors = Vec::new();
            return;
        }
    }

    // Número máximo de candidatos por ponto (constante ou n-1, o que for menor)
    let k = usize::min(crate::constants::CANDIDATE_LIST_SIZE, n.saturating_sub(1));
    
    // Caso especial: quando há 0 ou 1 ponto, não há vizinhos a considerar
    if n <= 1 {
        candidate_lists.nearest_neighbors = vec![Vec::new(); n];
        return;
    }
    
    // Pré-aloca o vetor com a capacidade exata para melhor performance
    let mut new_lists = Vec::with_capacity(n);
    
    for i in 0..n {
        // Coleta todos os vizinhos exceto o próprio ponto
        let mut neighbors = Vec::with_capacity(n - 1);
        
        for j in 0..n {
            if i != j {
                neighbors.push((j, distances.distances[i][j]));
            }
        }
        
        // Ordenação rápida por distância (sort_unstable é mais rápido que sort padrão)
        neighbors.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Extrai apenas os k vizinhos mais próximos
        let closest = neighbors.iter()
            .take(k)
            .map(|&(j, _)| j)
            .collect();
            
        new_lists.push(closest);
    }
    
    candidate_lists.nearest_neighbors = new_lists;
}
