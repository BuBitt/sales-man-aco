use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::time::{Duration, Instant};
use std::fmt::Write;
use crate::components::*;
use crate::resources::*;
use crate::ui::language::get_text;
use crate::utils::safe_despawn;
use crate::utils::point_distribution::generate_poisson_points;

/// Estima o tempo necessário para resolver o TSP usando um algoritmo de força bruta
/// 
/// Baseado na complexidade fatorial do problema do caixeiro viajante,
/// esta função calcula uma aproximação mais precisa de quanto tempo um algoritmo exato
/// levaria para encontrar a solução ótima.
/// 
/// # Argumentos
/// * `point_count` - O número de pontos/cidades no problema
/// 
/// # Retorno
/// Um `Duration` representando o tempo estimado de execução
fn estimate_standard_algorithm_time(point_count: usize) -> Duration {
    let n = point_count as f64;
    
    if n <= 3.0 {
        return Duration::from_millis(1);
    }
    
    // Algoritmo atualizado para estimativa mais precisa
    // Baseado em benchmarks reais de algoritmos exatos de TSP
    let operations_per_second = 1_000_000.0; // 1 milhão de operações/segundo em CPU moderno
    
    let mut factorial = 1.0;
    for i in 2..=point_count {
        factorial *= i as f64;
    }
    
    // Factor ajustado para refletir cálculos reais
    let seconds = if n <= 15.0 {
        factorial / operations_per_second * 0.000025
    } else {
        factorial / operations_per_second * 0.00001
    };
    
    // Prevenção contra overflow
    if seconds.is_infinite() || seconds > (std::u64::MAX as f64) {
        Duration::from_secs(std::u64::MAX)
    } else {
        Duration::from_secs_f64(seconds)
    }
}

/// Monitora e atualiza o estado de interação do usuário com a interface
/// 
/// # Argumentos
/// * `ui_state` - Estado atual da UI que será atualizado
/// * `contexts` - Contexto da interface EGUI
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
    let text = get_text(app_language.current);
    
    let ctx = contexts.ctx_mut();
    
    let mut visuals = ctx.style().visuals.clone();
    visuals.widgets.noninteractive.fg_stroke.width = 1.0;
    visuals.widgets.inactive.fg_stroke.width = 1.0;
    visuals.widgets.hovered.fg_stroke.width = 1.0;
    visuals.widgets.active.fg_stroke.width = 1.0;
    visuals.button_frame = true;
    
    ctx.set_visuals(visuals);

    egui::Window::new("TSP Controls")
        .default_width(320.0)
        .resizable(false)
        .show(ctx, |ui| {
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.strong(text.language);
            if ui.selectable_label(app_language.current == Language::English, "English").clicked() {
                app_language.current = Language::English;
            }
            if ui.selectable_label(app_language.current == Language::Portuguese, "Português").clicked() {
                app_language.current = Language::Portuguese;
            }
        });

        ui.add_space(10.0);
        
        // Point count control with simplified label
        ui.add(egui::Slider::new(&mut points.count, 5..=1000).text("Points"));
        
        // Add ant count control with simplified label
        ui.add(egui::Slider::new(&mut aco_params.ant_count, 5..=1000).text("Ants"));

        ui.add_space(8.0);
        
        // Center the generate points button
        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            let button_width = 180.0;
            let offset = (available_width - button_width) / 2.0;
            
            ui.add_space(offset);
            if ui.add_sized([button_width, 28.0], egui::Button::new(text.generate_points)).clicked() {
                entity_tracker.point_entities.clear();
                entity_tracker.path_entities.clear();
                
                for entity in point_markers.iter() {
                    safe_despawn(&mut commands, entity);
                }

                for entity in path_lines.iter() {
                    safe_despawn(&mut commands, entity);
                }

                for entity in best_path_lines.iter() {
                    safe_despawn(&mut commands, entity);
                }

                let count = points.count.max(5);
                points.count = count;

                let width = 900.0;
                let height = 700.0;
                
                points.positions = generate_poisson_points(count, width, height);

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
        });

        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);
        
        // Dynamic Start/Stop/Run Again button - centered with fixed size
        ui.horizontal(|ui| {
            let available_width = ui.available_width();
            let button_width = 160.0;
            let offset = (available_width - button_width) / 2.0;
            
            ui.add_space(offset);
            
            // Determine button text based on algorithm state
            let button_text = if aco_state.running { 
                text.stop 
            } else if aco_state.iterations > 0 && !points.positions.is_empty() { 
                // If algorithm has run but is stopped, and points exist, show "Run Again"
                text.run_again 
            } else { 
                text.start 
            };
            
            let mut button_response = ui.add_sized(
                [button_width, 32.0], 
                egui::Button::new(button_text)
            );
            
            if !aco_state.running && !points.positions.is_empty() {
                let tooltip = if aco_state.iterations > 0 {
                    if app_language.current == Language::English {
                        "Run the algorithm again with current parameters"
                    } else {
                        "Executar o algoritmo novamente com os parâmetros atuais"
                    }
                } else {
                    if app_language.current == Language::English {
                        "Press to start/restart the algorithm with current parameters"
                    } else {
                        "Pressione para iniciar/reiniciar o algoritmo com os parâmetros atuais"
                    }
                };
                button_response = button_response.on_hover_text(tooltip);
            }
            
            if button_response.clicked() {
                if aco_state.running {
                    aco_state.running = false;
                    if let Some(start_time) = aco_state.start_time {
                        aco_state.elapsed_time += Instant::now().duration_since(start_time);
                        aco_state.start_time = None;
                    }
                } else {
                    if !points.positions.is_empty() {
                        aco_state.iterations = 0;
                        aco_state.iterations_since_improvement = 0;
                        aco_state.elapsed_time = Duration::from_secs(0);
                        
                        best_path.path.clear();
                        best_path.distance = f32::INFINITY;
                        
                        let n = points.positions.len();
                        aco_state.pheromones = vec![vec![1.0; n]; n];
                        
                        entity_tracker.path_entities.clear();
                        
                        for entity in best_path_lines.iter() {
                            safe_despawn(&mut commands, entity);
                        }
                        
                        aco_state.running = true;
                        aco_state.start_time = Some(Instant::now());
                    }
                }
            }
        });

        ui.add_space(10.0);
        
        // Algorithm Parameters Category - Fix field name consistency
        egui::CollapsingHeader::new(text.algorithm_parameters)
            .default_open(false)
            .show(ui, |ui| {
                ui.add(egui::Slider::new(&mut aco_params.alpha, 0.1..=5.0).text(text.alpha));
                ui.add(egui::Slider::new(&mut aco_params.beta, 0.1..=5.0).text(text.beta));
                ui.add(egui::Slider::new(&mut aco_params.rho, 0.0..=1.0).text(text.rho));
                ui.add(egui::Slider::new(&mut aco_params.q, 1.0..=1000.0).text(text.q_factor));

                if ui.button(text.reset_parameters).clicked() {
                    *aco_params = AcoParameters::default();
                }
            });
            
        // Advanced Settings Category with safe implementation
        egui::CollapsingHeader::new(text.advanced_settings)
            .default_open(false)
            .show(ui, |ui| {
                ui.label("These settings affect algorithm behavior and performance:");
                ui.add_space(5.0);
                
                // Add candidate list size control with safe bounds
                let mut candidate_list_size = aco_state.candidate_list_size.unwrap_or(20);
                // Ensure candidate list size is always > 0 and <= max points
                let max_candidates = if points.positions.is_empty() { 50 } else { points.positions.len() - 1 };
                let max_candidates = max_candidates.max(5).min(50); // Keep within reasonable UI bounds
                
                if ui.add(egui::Slider::new(&mut candidate_list_size, 5..=max_candidates)
                    .text("Neighbor Candidates")).changed() {
                    aco_state.candidate_list_size = Some(candidate_list_size);
                }
                ui.add_space(2.0);
                
                // Add max iterations control
                let mut max_iterations = aco_state.max_iterations.unwrap_or(1000);
                if ui.add(egui::Slider::new(&mut max_iterations, 100..=5000).text("Max Iterations")).changed() {
                    aco_state.max_iterations = Some(max_iterations);
                }
                ui.add_space(2.0);
                
                // Add max iterations without improvement control
                let mut max_no_improvement = aco_state.max_iterations_no_improvement.unwrap_or(50);
                if ui.add(egui::Slider::new(&mut max_no_improvement, 10..=500).text("Max No Improvement")).changed() {
                    aco_state.max_iterations_no_improvement = Some(max_no_improvement);
                }
                
                ui.add_space(5.0);
                if ui.button("Reset Advanced Settings").clicked() {
                    aco_state.candidate_list_size = None;
                    aco_state.max_iterations = None;
                    aco_state.max_iterations_no_improvement = None;
                }
                
                ui.add_space(4.0);
                ui.label("Note: Changes take effect on next algorithm run.");
            });

        ui.add_space(5.0);
        
        // Stats section - keep at bottom
        if !points.positions.is_empty() {
            ui.label(text.positions_count.replace("{}", &points.positions.len().to_string()));

            let total_time = if let Some(start_time) = aco_state.start_time {
                aco_state.elapsed_time + Instant::now().duration_since(start_time)
            } else {
                aco_state.elapsed_time
            };

            let secs = total_time.as_secs();
            let millis = total_time.subsec_millis();
            
            let mut elapsed_str = String::new();
            write!(elapsed_str, "{:02}:{:02}.{:03}", secs / 60, secs % 60, millis).unwrap();
            ui.label(text.elapsed_time.replace("{:02}:{:02}.{:03}", &elapsed_str));

            let standard_time = estimate_standard_algorithm_time(points.positions.len());
            
            let time_display = if standard_time.as_secs() > 86400 * 365 * 1_000_000_000 {
                text.over_universe_age.to_string()
            } else if standard_time.as_secs() > 86400 * 365 * 1_000_000 {
                text.over_1000_millennia.to_string()
            } else if standard_time.as_secs() > 86400 * 365 * 10_000 {
                text.over_10_millennia.to_string()
            } else if standard_time.as_secs() > 86400 * 365 * 1_000 {
                text.over_a_millennium.to_string()
            } else if standard_time.as_secs() > 86400 * 365 * 100 {
                text.over_a_century.to_string()
            } else if standard_time.as_secs() > 86400 * 365 {
                text.over_a_year.to_string()
            } else if standard_time.as_secs() > 86400 * 30 {
                let months = standard_time.as_secs_f64() / (86400.0 * 30.0);
                text.months.replace("{:.1}", &format!("{:.1}", months))
            } else if standard_time.as_secs() > 86400 {
                let days = standard_time.as_secs_f64() / 86400.0;
                text.days.replace("{:.1}", &format!("{:.1}", days))
            } else if standard_time.as_secs() > 3600 {
                let hours = standard_time.as_secs_f64() / 3600.0;
                text.hours.replace("{:.1}", &format!("{:.1}", hours))
            } else if standard_time.as_secs() > 60 {
                let minutes = standard_time.as_secs_f64() / 60.0;
                text.minutes.replace("{:.1}", &format!("{:.1}", minutes))
            } else {
                let secs = standard_time.as_secs();
                let millis = standard_time.subsec_millis();
                text.seconds.replace("{}", &format!("{}", secs)).replace("{:03}", &format!("{:03}", millis))
            };
            
            ui.label(text.estimated_time_short.replace("{}", &time_display));
            ui.label(text.iterations.replace("{}", &aco_state.iterations.to_string()));

            if best_path.distance != f32::INFINITY {
                ui.label(text.best_distance.replace("{:.2}", &format!("{:.2}", best_path.distance)));
            }
        }
    });
}
