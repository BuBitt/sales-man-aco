use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rand::prelude::*;
use std::time::{Duration, Instant};
use crate::components::*;
use crate::resources::*;
use crate::ui::language::{get_text, UiText};
use crate::utils::{safe_despawn, safe_despawn_collection};

// Update this utility function to estimate standard algorithm time more accurately
fn estimate_standard_algorithm_time(point_count: usize) -> Duration {
    // For TSP, an exact algorithm like Held-Karp has O(n²·2ⁿ) complexity
    let n = point_count as f64;
    
    // Avoid overflow for large n by using a piecewise approach
    let estimated_seconds = if point_count <= 10 {
        // For small n, calculation is exact and quick
        let complexity = n * n * (2.0_f64.powf(n));
        let constant_factor = 0.000000001; // Nanoseconds per operation
        complexity * constant_factor
    } else if point_count <= 20 {
        // For medium n, use exponential growth model
        0.001 * (2.5_f64.powf(n))
    } else {
        // For large n, the time becomes astronomical - cap it reasonably
        // This represents "beyond practical computation"
        3600.0 * 24.0 * 365.0 // One year in seconds
    };
    
    Duration::from_secs_f64(estimated_seconds)
}

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
    mut app_language: ResMut<AppLanguage>,
    mut commands: Commands,
    mut entity_tracker: ResMut<EntityTracker>,
    point_markers: Query<Entity, With<PointMarker>>,
    path_lines: Query<Entity, With<PathLine>>,
    best_path_lines: Query<Entity, With<BestPathLine>>,
) {
    // Get the appropriate text based on the current language
    let text = get_text(app_language.current);
    
    egui::Window::new("TSP Controls").show(contexts.ctx_mut(), |ui| {
        // Language selector
        ui.horizontal(|ui| {
            ui.label(text.language);
            if ui.selectable_label(app_language.current == Language::English, "English").clicked() {
                app_language.current = Language::English;
            }
            if ui.selectable_label(app_language.current == Language::Portuguese, "Português").clicked() {
                app_language.current = Language::Portuguese;
            }
        });
        
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut points.count, 5..=100).text(text.points_slider));

            if ui.button(text.generate_points).clicked() {
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

            let button_text = if aco_state.running { text.stop } else { text.start };
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

        // Fix all format! calls to use explicit format strings
        ui.label(format!("{}", text.positions_count.replace("{}", &points.positions.len().to_string())));

        let total_time = if let Some(start_time) = aco_state.start_time {
            aco_state.elapsed_time + Instant::now().duration_since(start_time)
        } else {
            aco_state.elapsed_time
        };

        let secs = total_time.as_secs();
        let millis = total_time.subsec_millis();
        ui.label(format!("{}", text.elapsed_time.replace("{}", &format!("{:02}:{:02}.{:03}", secs / 60, secs % 60, millis))));

        // Format time more intelligently for very large durations
        if !points.positions.is_empty() {
            let standard_time = estimate_standard_algorithm_time(points.positions.len());
            
            // More intelligent formatting for different time scales
            let time_display = if standard_time.as_secs() > 86400 * 365 {
                text.over_a_year.to_string()
            } else if standard_time.as_secs() > 86400 * 30 {
                text.months.replace("{}", &format!("{:.1}", standard_time.as_secs_f64() / (86400.0 * 30.0)))
            } else if standard_time.as_secs() > 86400 {
                text.days.replace("{}", &format!("{:.1}", standard_time.as_secs_f64() / 86400.0))
            } else if standard_time.as_secs() > 3600 {
                text.hours.replace("{}", &format!("{:.1}", standard_time.as_secs_f64() / 3600.0))
            } else if standard_time.as_secs() > 60 {
                text.minutes.replace("{}", &format!("{:.1}", standard_time.as_secs_f64() / 60.0))
            } else {
                let secs = standard_time.as_secs();
                let millis = standard_time.subsec_millis();
                text.seconds.replace("{}", &format!("{:02}.{:03}", secs, millis))
            };
            
            ui.label(format!("{}", text.estimated_time.replace("{}", &time_display)));
                
            // Add comparison if we have current time and it makes sense
            if total_time.as_secs_f64() > 0.1 && standard_time.as_secs_f64() > 0.1 {
                let speedup = standard_time.as_secs_f64() / total_time.as_secs_f64();
                if speedup > 1.0 {
                    ui.label(format!("{}", text.aco_faster.replace("{}", &format!("{:.1}", speedup))));
                }
            }
        }

        ui.label(format!("{}", text.iterations.replace("{}", &aco_state.iterations.to_string())));

        if best_path.distance != f32::INFINITY {
            ui.label(format!("{}", text.best_distance.replace("{}", &format!("{:.2}", best_path.distance))));
        }

        ui.separator();
        ui.heading(text.algorithm_parameters);

        ui.add(egui::Slider::new(&mut aco_params.ant_count, 5..=100)
            .text(text.ant_count));
        
        if !points.positions.is_empty() {
            let num_points = points.positions.len();
            if aco_params.ant_count <= num_points {
                ui.label(format!("{}", text.each_ant_starts.replace("{}", &num_points.to_string())));
            } else {
                ui.label(format!("{}", text.ants_distributed.replace("{}", &num_points.to_string())));
            }
        }
        
        ui.add(egui::Slider::new(&mut aco_params.alpha, 0.1..=5.0).text(text.alpha));
        ui.add(egui::Slider::new(&mut aco_params.beta, 0.1..=5.0).text(text.beta));
        ui.add(egui::Slider::new(&mut aco_params.rho, 0.0..=1.0).text(text.rho));
        ui.add(egui::Slider::new(&mut aco_params.q, 1.0..=1000.0).text(text.q_factor));

        if ui.button(text.reset_parameters).clicked() {
            *aco_params = AcoParameters::default();
        }
    });
}
