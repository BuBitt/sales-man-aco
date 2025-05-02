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

        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut points.count, 5..=1000)
                .text(text.points_slider)
                .logarithmic(true));

            if ui.button(text.generate_points).clicked() {
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

            let button_text = if aco_state.running { text.stop } else { text.start };
            let mut button_response = ui.button(button_text);
            
            if !aco_state.running && !points.positions.is_empty() {
                let tooltip = if app_language.current == Language::English {
                    "Press to start/restart the algorithm with current parameters on the same points"
                } else {
                    "Pressione para iniciar/reiniciar o algoritmo com os parâmetros atuais nos mesmos pontos"
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

        if !points.positions.is_empty() {
            let standard_time = estimate_standard_algorithm_time(points.positions.len());
            
            let time_display = if standard_time.as_secs() > 86400 * 365 {
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
        }

        ui.label(text.iterations.replace("{}", &aco_state.iterations.to_string()));

        if best_path.distance != f32::INFINITY {
            ui.label(text.best_distance.replace("{:.2}", &format!("{:.2}", best_path.distance)));
        }

        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);

        ui.heading(text.algorithm_parameters);
        ui.add_space(10.0);

        ui.add(egui::Slider::new(&mut aco_params.ant_count, 5..=1000)
            .text(text.ant_count)
            .logarithmic(true));
        
        if !points.positions.is_empty() {
            let num_points = points.positions.len();
            if aco_params.ant_count <= num_points {
                ui.label(text.each_ant_starts.replace("{}", &num_points.to_string()));
            } else {
                ui.label(text.ants_distributed.replace("{}", &num_points.to_string()));
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
