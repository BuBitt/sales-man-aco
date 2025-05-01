use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use rand::prelude::*;
use rayon::prelude::*;
use std::time::{Duration, Instant};

const DEFAULT_ANT_COUNT: usize = 20;
const MAX_ITERATIONS: u32 = 1000;
const DEFAULT_ALPHA: f32 = 1.0; // Pheromone importance
const DEFAULT_BETA: f32 = 2.0;  // Distance importance
const DEFAULT_RHO: f32 = 0.5;   // Pheromone evaporation rate
const DEFAULT_Q: f32 = 100.0;   // Pheromone deposit factor

#[derive(Default, Resource)]
struct EntityTracker {
    point_entities: Vec<Entity>,
    path_entities: Vec<Entity>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "TSP - Ant Colony Optimization".into(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                }),
            EguiPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .insert_resource(AcoState::default())
        .insert_resource(Points::new())
        .insert_resource(BestPath::default())
        .insert_resource(AcoParameters::default())
        .insert_resource(UiState::default())
        .insert_resource(EntityTracker::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            ui_system,
            handle_input,
            camera_drag,
            camera_zoom,
            update_points_visualization,
            update_path_visualization,
            run_aco_algorithm,
        ))
        .run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Default, Resource)]
struct AcoState {
    running: bool,
    start_time: Option<Instant>,
    elapsed_time: Duration,
    iterations: u32,
    pheromones: Vec<Vec<f32>>,
}

#[derive(Default, Resource)]
struct Points {
    positions: Vec<Vec2>,
    count: usize,
}

impl Points {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            count: 20,
        }
    }
}

#[derive(Default, Resource)]
struct UiState {
    interacting_with_ui: bool,
}

#[derive(Resource)]
struct AcoParameters {
    ant_count: usize,
    alpha: f32,
    beta: f32,
    rho: f32,
    q: f32,
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

#[derive(Component)]
struct PointMarker;

#[derive(Component)]
struct PathLine;

#[derive(Component)]
struct BestPathLine;

#[derive(Default, Resource)]
struct BestPath {
    path: Vec<usize>,
    distance: f32,
}

#[derive(Component)]
struct Dragging;

struct Ant {
    visited: Vec<bool>,
    path: Vec<usize>,
    distance: f32,
}

impl Ant {
    fn new(point_count: usize) -> Self {
        Self {
            visited: vec![false; point_count],
            path: Vec::with_capacity(point_count),
            distance: 0.0,
        }
    }

    fn construct_solution(
        &mut self,
        points: &[Vec2],
        pheromones: &[Vec<f32>],
        rng: &mut ThreadRng,
        params: &AcoParameters,
    ) {
        let point_count = points.len();
        self.visited.fill(false);
        self.path.clear();
        self.distance = 0.0;

        let start = rng.gen_range(0..point_count);
        self.path.push(start);
        self.visited[start] = true;

        while self.path.len() < point_count {
            let current = *self.path.last().unwrap();
            let next = self.select_next_city(current, points, pheromones, rng, params);

            let distance = points[current].distance(points[next]);
            self.distance += distance;

            self.path.push(next);
            self.visited[next] = true;
        }

        let first = self.path[0];
        let last = self.path[point_count - 1];
        self.distance += points[last].distance(points[first]);
    }

    fn select_next_city(
        &self,
        current: usize,
        points: &[Vec2],
        pheromones: &[Vec<f32>],
        rng: &mut ThreadRng,
        params: &AcoParameters,
    ) -> usize {
        let point_count = points.len();

        let mut total_prob = 0.0;
        let mut probabilities = vec![0.0; point_count];

        for i in 0..point_count {
            if !self.visited[i] {
                let distance = points[current].distance(points[i]);
                let pheromone = pheromones[current][i];

                let distance_factor = if distance < 0.0001 { 1000.0 } else { 1.0 / distance };

                probabilities[i] = pheromone.powf(params.alpha) * distance_factor.powf(params.beta);
                total_prob += probabilities[i];
            }
        }

        let mut choice = rng.gen::<f32>() * total_prob;
        for i in 0..point_count {
            if !self.visited[i] {
                choice -= probabilities[i];
                if choice <= 0.0 {
                    return i;
                }
            }
        }

        for i in 0..point_count {
            if !self.visited[i] {
                return i;
            }
        }

        unreachable!("Should have found an unvisited city")
    }
}

fn safe_despawn_collection(
    commands: &mut Commands,
    entities: &mut Vec<Entity>,
) {
    let mut to_remove = Vec::new();
    for (i, &entity) in entities.iter().enumerate() {
        if commands.get_entity(entity).is_some() {
            commands.entity(entity).despawn_recursive();
            to_remove.push(i);
        }
    }

    to_remove.sort_by(|a, b| b.cmp(a));
    for index in to_remove {
        entities.remove(index);
    }
}

fn safe_despawn(commands: &mut Commands, entity: Entity) {
    if commands.get_entity(entity).is_some() {
        commands.entity(entity).despawn_recursive();
    }
}

fn calculate_distance(path: &[usize], points: &[Vec2]) -> f32 {
    let mut distance = 0.0;
    let len = path.len();

    for i in 0..len - 1 {
        distance += points[path[i]].distance(points[path[i + 1]]);
    }

    distance += points[path[len - 1]].distance(points[path[0]]);
    distance
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
    ));

    commands.spawn(
        TextBundle::from_section(
            "Drag to pan, scroll to zoom",
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

fn handle_input(
    mut ui_state: ResMut<UiState>,
    mut contexts: EguiContexts,
) {
    ui_state.interacting_with_ui = contexts.ctx_mut().is_using_pointer();
}

fn ui_system(
    mut contexts: EguiContexts,
    mut aco_state: ResMut<AcoState>,
    mut points: ResMut<Points>,
    mut best_path: ResMut<BestPath>,
    mut aco_params: ResMut<AcoParameters>,
    mut commands: Commands,
    mut entity_tracker: ResMut<EntityTracker>,
    point_markers: Query<Entity, With<PointMarker>>,
    path_lines: Query<Entity, With<PathLine>>,
    best_path_lines: Query<Entity, With<BestPathLine>>,
) {
    egui::Window::new("TSP Controls").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut points.count, 5..=100).text("Points"));

            if ui.button("Generate Points").clicked() {
                for entity in point_markers.iter() {
                    safe_despawn(&mut commands, entity);
                }

                for entity in path_lines.iter() {
                    safe_despawn(&mut commands, entity);
                }

                for entity in best_path_lines.iter() {
                    safe_despawn(&mut commands, entity);
                }

                entity_tracker.point_entities.clear();
                entity_tracker.path_entities.clear();

                let count = points.count.max(5);
                points.count = count;

                let mut rng = thread_rng();
                points.positions = (0..count)
                    .map(|_| Vec2::new(
                        rng.gen_range(-400.0..400.0),
                        rng.gen_range(-300.0..300.0),
                    ))
                    .collect();

                aco_state.running = false;
                aco_state.iterations = 0;
                aco_state.start_time = None;
                aco_state.elapsed_time = Duration::from_secs(0);
                best_path.path.clear();
                best_path.distance = f32::INFINITY;

                let n = points.positions.len();
                aco_state.pheromones = vec![vec![1.0; n]; n];
            }

            let button_text = if aco_state.running { "Stop" } else { "Start" };
            if ui.button(button_text).clicked() {
                aco_state.running = !aco_state.running;
                if aco_state.running {
                    aco_state.start_time = Some(Instant::now());
                } else if let Some(start_time) = aco_state.start_time {
                    aco_state.elapsed_time += Instant::now().duration_since(start_time);
                    aco_state.start_time = None;
                }
            }
        });

        ui.label(format!("Positions count: {}", points.positions.len()));

        let total_time = if let Some(start_time) = aco_state.start_time {
            aco_state.elapsed_time + Instant::now().duration_since(start_time)
        } else {
            aco_state.elapsed_time
        };

        let secs = total_time.as_secs();
        let millis = total_time.subsec_millis();
        ui.label(format!("Elapsed: {:02}:{:02}.{:03}", secs / 60, secs % 60, millis));

        ui.label(format!("Iterations: {}", aco_state.iterations));

        if best_path.distance != f32::INFINITY {
            ui.label(format!("Best distance: {:.2}", best_path.distance));
        }

        ui.separator();
        ui.heading("Algorithm Parameters");

        ui.add(egui::Slider::new(&mut aco_params.ant_count, 5..=100)
            .text("Ant Count"));
        
        if !points.positions.is_empty() {
            let num_points = points.positions.len();
            if aco_params.ant_count <= num_points {
                ui.label(format!("Cada formiga iniciará em um ponto distinto (total: {})", num_points));
            } else {
                ui.label(format!("Formigas serão distribuídas aleatoriamente entre os {} pontos", num_points));
            }
        }
        
        ui.add(egui::Slider::new(&mut aco_params.alpha, 0.1..=5.0).text("Alpha (pheromone importance)"));
        ui.add(egui::Slider::new(&mut aco_params.beta, 0.1..=5.0).text("Beta (distance importance)"));
        ui.add(egui::Slider::new(&mut aco_params.rho, 0.0..=1.0).text("Rho (evaporation rate)"));
        ui.add(egui::Slider::new(&mut aco_params.q, 1.0..=1000.0).text("Q (pheromone deposit)"));

        if ui.button("Reset Parameters").clicked() {
            *aco_params = AcoParameters::default();
        }
    });
}

fn camera_drag(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(Entity, &mut Transform, Option<&Dragging>), With<MainCamera>>,
    ui_state: Res<UiState>,
) {
    let (camera_entity, mut camera_transform, dragging) = query.single_mut();

    if mouse_buttons.just_pressed(MouseButton::Left) && !ui_state.interacting_with_ui {
        commands.entity(camera_entity).insert(Dragging);
    }

    if mouse_buttons.just_released(MouseButton::Left) || ui_state.interacting_with_ui {
        if dragging.is_some() {
            commands.entity(camera_entity).remove::<Dragging>();
        }
    }

    if dragging.is_some() && !ui_state.interacting_with_ui {
        for event in mouse_motion_events.read() {
            camera_transform.translation.x -= event.delta.x * camera_transform.scale.x;
            camera_transform.translation.y += event.delta.y * camera_transform.scale.y;
        }
    }
}

fn camera_zoom(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<MainCamera>>,
    ui_state: Res<UiState>,
) {
    if ui_state.interacting_with_ui {
        return;
    }

    let mut camera_transform = query.single_mut();

    for event in mouse_wheel_events.read() {
        let zoom_factor = if event.y > 0.0 { 0.9 } else { 1.1 };

        camera_transform.scale.x = (camera_transform.scale.x * zoom_factor).clamp(0.1, 5.0);
        camera_transform.scale.y = (camera_transform.scale.y * zoom_factor).clamp(0.1, 5.0);
    }
}

fn update_points_visualization(
    mut commands: Commands,
    points: Res<Points>,
    mut entity_tracker: ResMut<EntityTracker>,
    point_markers: Query<Entity, With<PointMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if (!points.is_changed() && !point_markers.is_empty() && !entity_tracker.point_entities.is_empty()) 
        || points.positions.is_empty() {
        return;
    }

    for entity in point_markers.iter() {
        safe_despawn(&mut commands, entity);
    }

    entity_tracker.point_entities.clear();

    let circle = meshes.add(Circle::new(5.0));
    let material = materials.add(ColorMaterial::from(Color::WHITE));

    for position in &points.positions {
        let entity = commands.spawn((
            MaterialMesh2dBundle {
                mesh: circle.clone().into(),
                material: material.clone(),
                transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
                ..default()
            },
            PointMarker,
        )).id();
        
        entity_tracker.point_entities.push(entity);
    }
}

fn update_path_visualization(
    mut commands: Commands,
    best_path: Res<BestPath>,
    points: Res<Points>,
    mut entity_tracker: ResMut<EntityTracker>,
    best_path_lines: Query<Entity, With<BestPathLine>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !best_path.is_changed() || best_path.path.is_empty() {
        return;
    }

    safe_despawn_collection(&mut commands, &mut entity_tracker.path_entities);

    for entity in best_path_lines.iter() {
        safe_despawn(&mut commands, entity);
    }

    let n = best_path.path.len();
    let _line_material = materials.add(ColorMaterial::from(Color::srgb(0.0, 1.0, 0.0)));

    entity_tracker.path_entities.clear();
    for i in 0..n {
        let from_idx = best_path.path[i];
        let to_idx = best_path.path[(i + 1) % n];

        if from_idx >= points.positions.len() || to_idx >= points.positions.len() {
            continue;
        }

        let from = points.positions[from_idx];
        let to = points.positions[to_idx];
        let direction = to - from;
        let length = direction.length();

        let entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    custom_size: Some(Vec2::new(length, 1.5)),
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
        
        entity_tracker.path_entities.push(entity);
    }
}

fn run_aco_algorithm(
    mut aco_state: ResMut<AcoState>,
    points: Res<Points>,
    mut best_path: ResMut<BestPath>,
    aco_params: Res<AcoParameters>,
) {
    if !aco_state.running || points.positions.is_empty() {
        return;
    }

    if aco_state.iterations >= MAX_ITERATIONS {
        aco_state.running = false;
        if let Some(start_time) = aco_state.start_time {
            aco_state.elapsed_time += Instant::now().duration_since(start_time);
            aco_state.start_time = None;
        }
        return;
    }

    aco_state.iterations += 1;
    let n = points.positions.len();

    let mut ants: Vec<Ant> = (0..aco_params.ant_count).map(|_| Ant::new(n)).collect();

    if aco_params.ant_count <= n {
        let mut starting_points: Vec<usize> = (0..n).collect();
        let mut rng = thread_rng();
        starting_points.shuffle(&mut rng);

        ants.par_iter_mut().enumerate().for_each(|(i, ant)| {
            let mut local_rng = thread_rng();
            let start = starting_points[i];
            ant.visited.fill(false);
            ant.path.clear();
            ant.distance = 0.0;
            
            ant.path.push(start);
            ant.visited[start] = true;
            
            while ant.path.len() < n {
                let current = *ant.path.last().unwrap();
                let next = ant.select_next_city(current, &points.positions, &aco_state.pheromones, &mut local_rng, &aco_params);
                
                let distance = points.positions[current].distance(points.positions[next]);
                ant.distance += distance;
                
                ant.path.push(next);
                ant.visited[next] = true;
            }
            
            let first = ant.path[0];
            let last = ant.path[n - 1];
            ant.distance += points.positions[last].distance(points.positions[first]);
        });
    } else {
        ants.par_iter_mut().for_each(|ant| {
            let mut local_rng = thread_rng();
            ant.construct_solution(&points.positions, &aco_state.pheromones, &mut local_rng, &aco_params);
        });
    }

    let mut iteration_best_ant = &ants[0];
    for ant in &ants {
        if ant.distance < iteration_best_ant.distance {
            iteration_best_ant = ant;
        }
    }

    let new_best_distance = calculate_distance(&iteration_best_ant.path, &points.positions);
    if new_best_distance < best_path.distance {
        best_path.path = iteration_best_ant.path.clone();
        best_path.distance = new_best_distance;
    }

    for i in 0..n {
        for j in 0..n {
            aco_state.pheromones[i][j] *= 1.0 - aco_params.rho;
        }
    }

    for ant in &ants {
        let pheromone_amount = aco_params.q / ant.distance;
        for i in 0..n - 1 {
            let from = ant.path[i];
            let to = ant.path[i + 1];
            aco_state.pheromones[from][to] += pheromone_amount;
            aco_state.pheromones[to][from] += pheromone_amount;
        }

        let from = ant.path[n - 1];
        let to = ant.path[0];
        aco_state.pheromones[from][to] += pheromone_amount;
        aco_state.pheromones[to][from] += pheromone_amount;
    }
}
