use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rand::prelude::*;
use std::time::{Duration, Instant};
use crate::components::*;
use crate::resources::*;
use crate::utils::{safe_despawn, safe_despawn_collection};

pub fn handle_input(
    mut ui_state: ResMut<UiState>,
    mut contexts: EguiContexts,
) {
    ui_state.interacting_with_ui = contexts.ctx_mut().is_using_pointer();
}

pub fn ui_system(
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
                safe_despawn_collection(&mut commands, &mut entity_tracker.point_entities);
                safe_despawn_collection(&mut commands, &mut entity_tracker.path_entities);

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
                aco_state.iterations_since_improvement = 0;
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
