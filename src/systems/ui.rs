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
    // Capture both direct interaction and hovering state
    let ctx = contexts.ctx_mut();
    ui_state.interacting_with_ui = ctx.is_using_pointer() || ctx.is_pointer_over_area();
    ui_state.hovering_ui = ctx.is_pointer_over_area();
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
    
    // Apply a dark mode visual style
    let mut visuals = egui::style::Visuals::dark();
    // Configure shadow for dark theme
    visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::Vec2::new(2.0, 4.0),
        blur: 8.0,
        spread: 0.0,
        color: egui::Color32::from_black_alpha(70), // Darker shadow for dark mode
    };
    visuals.widgets.noninteractive.fg_stroke.width = 1.0;
    visuals.widgets.inactive.fg_stroke.width = 1.0;
    visuals.widgets.hovered.fg_stroke.width = 1.5;
    visuals.widgets.active.fg_stroke.width = 1.5;
    visuals.window_rounding = egui::Rounding::same(6.0);
    // Dark background with slight blue tint
    visuals.window_fill = egui::Color32::from_rgb(25, 27, 38);
    
    // Custom text styles for dark mode - slightly larger for better readability
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Button, 
        egui::FontId::new(15.0, egui::FontFamily::Proportional)
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(18.0, egui::FontFamily::Proportional)
    );
    
    ctx.set_style(style);
    ctx.set_visuals(visuals);

    // Main panel with contemporary design
    egui::SidePanel::left("tsp_controls_panel")
        .default_width(360.0)
        .resizable(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                // App title with better styling
                ui.heading("Traveling Salesman Solver");
                ui.add_space(2.0);
            });
            
            // Language selection in a more subtle location
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                    if ui.selectable_label(app_language.current == Language::Portuguese, "Português").clicked() {
                        app_language.current = Language::Portuguese;
                    }
                    if ui.selectable_label(app_language.current == Language::English, "English").clicked() {
                        app_language.current = Language::English;
                    }
                    ui.label(text.language);
                });
            });
            
            ui.add_space(8.0);
            ui.separator();
            
            // Configure ScrollArea to handle content better
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(10.0);
                    
                    // SECTION 1: PROBLEM SETUP - Using existing text field
                    ui.vertical(|ui| {
                        ui.add(egui::widgets::Label::new(egui::RichText::new(text.basic_settings)
                            .strong()
                            .size(16.0)));
                        
                        ui.add_space(6.0);
                        
                        // Point count control
                        ui.add(egui::Slider::new(&mut points.count, 5..=1000).text(text.points_slider))
                            .on_hover_text("Quantidade de pontos (cidades) para o problema");
                        
                        ui.add_space(4.0);
                        
                        // Generate button with improved styling
                        if ui.add_sized([ui.available_width(), 32.0], 
                            egui::Button::new(egui::RichText::new(text.generate_points)
                                .text_style(egui::TextStyle::Button)))
                            .clicked() 
                        {
                            // Reset points and clear display
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
                    
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);
                    
                    // SECTION 2: ALGORITHM EXECUTION - Using algorithm_parameters as section title
                    ui.vertical(|ui| {
                        ui.add(egui::widgets::Label::new(egui::RichText::new(text.algorithm_parameters)
                            .strong()
                            .size(16.0)));
                        
                        ui.add_space(6.0);

                        // Ant count slider
                        ui.add(egui::Slider::new(&mut aco_params.ant_count, 5..=1000).text(text.ant_count))
                            .on_hover_text("Número de formigas para explorar soluções");
                        
                        ui.add_space(4.0);
                        
                        // Execution button with improved styling for dark mode
                        let button_text = if aco_state.running { 
                            text.stop 
                        } else if aco_state.iterations > 0 && !points.positions.is_empty() { 
                            text.run_again 
                        } else { 
                            text.start 
                        };
                        
                        // Define both background and text colors for better visibility
                        let (bg_color, text_color) = if aco_state.running {
                            (egui::Color32::from_rgb(150, 30, 30), egui::Color32::from_rgb(255, 200, 200)) // Red bg, light text
                        } else if aco_state.iterations > 0 {
                            (egui::Color32::from_rgb(30, 100, 30), egui::Color32::from_rgb(200, 255, 200)) // Green bg, light text
                        } else {
                            (egui::Color32::from_rgb(30, 60, 120), egui::Color32::from_rgb(200, 220, 255)) // Blue bg, light text
                        };
                        
                        // Replace the custom frame with direct button styling
                        if ui.add_sized(
                            [ui.available_width(), 32.0], // Full width, fixed height
                            egui::Button::new(
                                egui::RichText::new(button_text)
                                    .text_style(egui::TextStyle::Button)
                                    .color(text_color)
                            )
                            .fill(bg_color) // Apply background color directly to button
                            .rounding(egui::Rounding::same(4.0))
                        ).clicked() {
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
                    
                    // SECTION 3: STATISTICS SECTION - If we have points and data
                    if !points.positions.is_empty() {
                        ui.add_space(12.0);
                        
                        // Update statistics panel with dark mode compatible colors
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgba_premultiplied(40, 42, 54, 200))  // Dark background that matches theme
                            .rounding(egui::Rounding::same(4.0))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(100)))  // Lighter border
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.add_space(8.0); // Increased top padding
                                    
                                    // Add horizontal padding to all content
                                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                        // Title with padding
                                        ui.add(egui::widgets::Label::new(
                                            egui::RichText::new("Statistics").strong().size(15.0)
                                        ));
                                    });
                                    
                                    ui.add_space(8.0); // Increased spacing after title
                                    
                                    // Wrap grid in a container with horizontal padding
                                    ui.horizontal(|ui| {
                                        ui.add_space(12.0); // Left padding
                                        
                                        ui.vertical(|ui| {
                                            // Statistics grid for better alignment
                                            egui::Grid::new("stats_grid")
                                                .num_columns(2)
                                                .spacing([8.0, 4.0])
                                                .striped(true)
                                                .show(ui, |ui| {
                                                    // Points count
                                                    ui.label(egui::RichText::new("Points:").strong());
                                                    ui.label(points.positions.len().to_string());
                                                    ui.end_row();
                                                    
                                                    // Elapsed time
                                                    let total_time = if let Some(start_time) = aco_state.start_time {
                                                        aco_state.elapsed_time + Instant::now().duration_since(start_time)
                                                    } else {
                                                        aco_state.elapsed_time
                                                    };
                                                    
                                                    let secs = total_time.as_secs();
                                                    let millis = total_time.subsec_millis();
                                                    
                                                    let mut elapsed_str = String::new();
                                                    write!(elapsed_str, "{:02}:{:02}.{:03}", secs / 60, secs % 60, millis).unwrap();
                                                    
                                                    ui.label(egui::RichText::new("Time:").strong());
                                                    ui.label(elapsed_str);
                                                    ui.end_row();
                                                    
                                                    // Estimated time - using existing calculation
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
                                                    
                                                    ui.label(egui::RichText::new("Brute force:").strong());
                                                    ui.label(time_display);
                                                    ui.end_row();
                                                    
                                                    // Iterations
                                                    ui.label(egui::RichText::new("Iterations:").strong());
                                                    ui.label(aco_state.iterations.to_string());
                                                    ui.end_row();
                                                    
                                                    // Best distance if available
                                                    if best_path.distance != f32::INFINITY {
                                                        ui.label(egui::RichText::new("Best distance:").strong());
                                                        ui.label(format!("{:.2}", best_path.distance));
                                                        ui.end_row();
                                                    }
                                                });
                                        });
                                        
                                        ui.add_space(12.0); // Right padding
                                    });
                                    
                                    ui.add_space(8.0); // Increased bottom padding
                                });
                            });
                    }
                    
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    
                    // SECTION 4: ALGORITHM PARAMETERS (Collapsible) - Using a more specific name
                    egui::CollapsingHeader::new(
                        egui::RichText::new("Algorithm Parameters")
                            .strong()
                            .size(16.0)
                    )
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        
                        // Enhanced parameter sliders with better descriptions
                        ui.add(egui::Slider::new(&mut aco_params.alpha, 0.1..=5.0).text(text.alpha))
                            .on_hover_text("Alpha: Controls the influence of pheromone trails.\n\
                                          Higher values = ants follow existing paths more closely.\n\
                                          Lower values = more exploration of new paths.");
                        
                        ui.add_space(4.0);
                        ui.add(egui::Slider::new(&mut aco_params.beta, 0.1..=5.0).text(text.beta))
                            .on_hover_text("Beta: Controls the influence of distance between cities.\n\
                                          Higher values = ants prefer shorter distances.\n\
                                          Lower values = less greedy, more exploration.");
                        
                        ui.add_space(4.0);
                        ui.add(egui::Slider::new(&mut aco_params.rho, 0.0..=1.0).text(text.rho))
                            .on_hover_text("Rho: Pheromone evaporation rate.\n\
                                          Higher values = faster forgetting of poorer paths.\n\
                                          Lower values = more persistence of historical information.");
                        
                        ui.add_space(4.0);
                        ui.add(egui::Slider::new(&mut aco_params.q, 1.0..=1000.0).text(text.q_factor))
                            .on_hover_text("Q: Pheromone deposit quantity factor.\n\
                                          Controls how much pheromone ants deposit on paths.\n\
                                          Higher values = stronger reinforcement of good paths.");

                        ui.add_space(8.0);
                        if ui.button(text.reset_parameters).clicked() {
                            *aco_params = AcoParameters::default();
                        }
                    });
                    
                    ui.add_space(8.0);
                    
                    // SECTION 5: ADVANCED SETTINGS (Collapsible)
                    egui::CollapsingHeader::new(
                        egui::RichText::new(text.advanced_settings)
                            .strong()
                            .size(16.0)
                    )
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label("These settings affect algorithm behavior and performance:");
                        ui.add_space(5.0);
                        
                        // Performance Mode toggle
                        let mut performance_mode = aco_state.performance_mode.unwrap_or(false);
                        let perf_checkbox = ui.checkbox(&mut performance_mode, "High Performance Mode");
                        if perf_checkbox.changed() {
                            aco_state.performance_mode = Some(performance_mode);
                        }
                        perf_checkbox.on_hover_text(
                            "Optimizes computation and rendering to improve framerate with many points.\n\
                            Recommended for problems with 100+ points or older hardware."
                        );
                        
                        if performance_mode {
                            ui.label("Reduces computational load for better performance");
                        }
                        ui.add_space(5.0);
                        
                        // Add adaptive ant count feature
                        let auto_ant_count = if points.positions.len() > 100 {
                            // For large problems, use fewer ants (square root scaling)
                            (points.positions.len() as f32).sqrt().max(10.0) as usize
                        } else {
                            points.positions.len()
                        };
                        
                        // Ant count control with performance warning
                        ui.horizontal(|ui| {
                            ui.label("Ant Count:")
                                .on_hover_text("Number of ants exploring the graph in each iteration.\n\
                                              More ants = better solutions but slower performance.\n\
                                              For large problems, 30-50 ants is often sufficient.");
                            if ui.small_button("Auto").clicked() {
                                aco_params.ant_count = auto_ant_count;
                            }
                        });
                        
                        let prev_ant_count = aco_params.ant_count;
                        ui.add(egui::Slider::new(&mut aco_params.ant_count, 5..=1000).text(""))
                            .on_hover_text("Ant count significantly impacts performance.\n\
                                           Complexity: O(iterations × ants × points²)");
                        
                        // Show performance impact warning for large ant counts
                        if aco_params.ant_count >= 100 && points.positions.len() >= 50 {
                            ui.label(format!("⚠️ High ant count may reduce performance"));
                            if prev_ant_count != aco_params.ant_count {
                                // Recommend optimal ant count when user changes the value
                                ui.label(format!("Recommended: {} ants for {} points", 
                                                auto_ant_count, points.positions.len()));
                            }
                        }
                        
                        // Visualization frequency for large problems
                        if points.positions.len() > 50 || aco_params.ant_count > 50 {
                            ui.add_space(5.0);
                            let mut viz_frequency = aco_state.visualization_frequency.unwrap_or(1);
                            ui.horizontal(|ui| {
                                ui.label("Visualization Rate:")
                                    .on_hover_text("How often the display updates during computation.\n\
                                                  Higher values = smoother UI but less visual feedback.");
                                if ui.small_button("Auto").clicked() {
                                    // Automatically adjust visualization frequency based on problem size
                                    viz_frequency = if points.positions.len() * aco_params.ant_count > 10000 {
                                        10
                                    } else if points.positions.len() * aco_params.ant_count > 5000 {
                                        5
                                    } else {
                                        1
                                    };
                                    aco_state.visualization_frequency = Some(viz_frequency);
                                }
                            });
                            
                            if ui.add(egui::Slider::new(&mut viz_frequency, 1..=20).text("")).changed() {
                                aco_state.visualization_frequency = Some(viz_frequency);
                            }
                            ui.label(format!("Update every {} iterations (higher = faster)", viz_frequency));
                        }
                        
                        // Parallel execution options
                        let mut parallel_ants = aco_state.parallel_ants.unwrap_or(true);
                        let parallel_checkbox = ui.checkbox(&mut parallel_ants, "Parallel Ant Processing");
                        if parallel_checkbox.changed() {
                            aco_state.parallel_ants = Some(parallel_ants);
                        }
                        parallel_checkbox.on_hover_text(
                            "Uses multiple CPU cores to process ants in parallel.\n\
                            Disable on very old hardware if you experience issues."
                        );
                        
                        // Add candidate list size control with safe bounds
                        let mut candidate_list_size = aco_state.candidate_list_size.unwrap_or(20);
                        // Ensure candidate list size is always > 0 and <= max points
                        let max_candidates = if points.positions.is_empty() { 50 } else { points.positions.len() - 1 };
                        let max_candidates = max_candidates.max(5).min(50); // Keep within reasonable UI bounds
                        
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label("Neighbor Candidates:")
                                .on_hover_text("The number of closest cities each ant considers at each step.\n\
                                               Smaller values = faster computation but potentially lower quality solutions.\n\
                                               Larger values = better solutions but slower computation.");
                            if ui.small_button("Auto").clicked() {
                                // For large problems, use smaller candidate list
                                candidate_list_size = if points.positions.len() > 100 {
                                    (points.positions.len() as f32 * 0.1).max(5.0).min(20.0) as usize
                                } else {
                                    20
                                };
                                aco_state.candidate_list_size = Some(candidate_list_size);
                            }
                        });
                        
                        if ui.add(egui::Slider::new(&mut candidate_list_size, 5..=max_candidates)
                            .text("")).changed() {
                            aco_state.candidate_list_size = Some(candidate_list_size);
                        }
                        
                        ui.add_space(2.0);
                        
                        // Add max iterations control
                        ui.label("Max Iterations:")
                            .on_hover_text("Maximum number of algorithm iterations before stopping.\n\
                                           Higher values allow finding better solutions but take longer.");
                        let mut max_iterations = aco_state.max_iterations.unwrap_or(1000);
                        if ui.add(egui::Slider::new(&mut max_iterations, 100..=5000).text("")).changed() {
                            aco_state.max_iterations = Some(max_iterations);
                        }
                        ui.add_space(2.0);
                        
                        // Add max iterations without improvement control
                        ui.label("Max No Improvement:")
                            .on_hover_text("Stops the algorithm early if no better solution is found after this many iterations.\n\
                                           Lower values = faster termination, higher values = more thorough search.");
                        let mut max_no_improvement = aco_state.max_iterations_no_improvement.unwrap_or(50);
                        if ui.add(egui::Slider::new(&mut max_no_improvement, 10..=500).text("")).changed() {
                            aco_state.max_iterations_no_improvement = Some(max_no_improvement);
                        }
                        
                        ui.add_space(5.0);
                        if ui.button("Reset Advanced Settings").clicked() {
                            aco_state.candidate_list_size = None;
                            aco_state.max_iterations = None;
                            aco_state.max_iterations_no_improvement = None;
                            aco_state.performance_mode = None;
                            aco_state.visualization_frequency = None;
                            aco_state.parallel_ants = None;
                        }
                        
                        // Performance explanation section
                        if points.positions.len() > 50 || aco_params.ant_count > 50 {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(5.0);
                            ui.label("Performance Optimization Tips:");
                            ui.label("• Time complexity: O(iterations × ants × points²)");
                            ui.label("• Reduce ant count for large problems (30-50 ants is often sufficient)");
                            ui.label("• Enable High Performance Mode for problems with 100+ points");
                            ui.label("• Increase visualization rate to 5-10 for smoother UI");
                            ui.label("• Use smaller candidate lists (reduces computation per ant)");
                        }
                    });
                });
        });
}
