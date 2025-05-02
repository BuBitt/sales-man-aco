use crate::resources::Language;

pub struct UiText {
    pub language: &'static str,
    pub basic_settings: &'static str,
    pub algorithm_parameters: &'static str,
    pub advanced_settings: &'static str,
    pub advanced_settings_description: &'static str,
    
    pub points_slider: &'static str,
    pub ant_count: &'static str,
    pub generate_points: &'static str,
    
    pub start: &'static str,
    pub stop: &'static str,
    pub run_again: &'static str,
    pub reset_parameters: &'static str,
    
    pub alpha: &'static str,
    pub beta: &'static str,
    pub rho: &'static str,
    pub q_factor: &'static str,
    
    pub positions_count: &'static str,
    pub elapsed_time: &'static str,
    pub estimated_time_short: &'static str,
    pub iterations: &'static str,
    pub best_distance: &'static str,
    
    pub over_universe_age: &'static str,
    pub over_1000_millennia: &'static str,
    pub over_10_millennia: &'static str,
    pub over_a_millennium: &'static str,
    pub over_a_century: &'static str,
    pub over_a_year: &'static str,
    pub months: &'static str,
    pub days: &'static str,
    pub hours: &'static str,
    pub minutes: &'static str,
    pub seconds: &'static str,
}

pub fn get_text(language: Language) -> UiText {
    match language {
        Language::English => UiText {
            language: "Language",
            basic_settings: "Basic Settings",
            algorithm_parameters: "Algorithm Parameters",
            advanced_settings: "Advanced Settings",
            advanced_settings_description: "More settings will be available in future updates.",
            
            points_slider: "Number of Points",
            ant_count: "Number of Ants",
            generate_points: "Generate Points",
            
            start: "Start",
            stop: "Stop",
            run_again: "Run Again",
            reset_parameters: "Reset Parameters",
            
            alpha: "Alpha (Pheromone Weight)",
            beta: "Beta (Distance Weight)",
            rho: "Rho (Evaporation Rate)",
            q_factor: "Q (Pheromone Quantity)",
            
            positions_count: "Points: {}",
            elapsed_time: "Time: {:02}:{:02}.{:03}",
            estimated_time_short: "Est. time with brute force: {}",
            iterations: "Iterations: {}",
            best_distance: "Best distance: {:.2}",
            
            over_universe_age: "over universe age",
            over_1000_millennia: "over 1000 millennia",
            over_10_millennia: "over 10 millennia",
            over_a_millennium: "over a millennium",
            over_a_century: "over a century",
            over_a_year: "over a year",
            months: "{:.1} months",
            days: "{:.1} days",
            hours: "{:.1} hours",
            minutes: "{:.1} minutes",
            seconds: "{} seconds {.3} ms",
        },
        Language::Portuguese => UiText {
            language: "Idioma",
            basic_settings: "Configurações Básicas",
            algorithm_parameters: "Parâmetros do Algoritmo",
            advanced_settings: "Configurações Avançadas",
            advanced_settings_description: "Mais configurações estarão disponíveis em atualizações futuras.",
            
            points_slider: "Número de Pontos",
            ant_count: "Número de Formigas",
            generate_points: "Gerar Pontos",
            
            start: "Iniciar",
            stop: "Parar",
            run_again: "Executar Novamente",
            reset_parameters: "Redefinir Parâmetros",
            
            alpha: "Alpha (Peso do Feromônio)",
            beta: "Beta (Peso da Distância)",
            rho: "Rho (Taxa de Evaporação)",
            q_factor: "Q (Quantidade de Feromônio)",
            
            positions_count: "Pontos: {}",
            elapsed_time: "Tempo: {:02}:{:02}.{:03}",
            estimated_time_short: "Tempo estimado com força bruta: {}",
            iterations: "Iterações: {}",
            best_distance: "Melhor distância: {:.2}",
            
            over_universe_age: "mais que a idade do universo",
            over_1000_millennia: "mais de 1000 milênios",
            over_10_millennia: "mais de 10 milênios",
            over_a_millennium: "mais de um milênio",
            over_a_century: "mais de um século",
            over_a_year: "mais de um ano",
            months: "{:.1} meses",
            days: "{:.1} dias",
            hours: "{:.1} horas",
            minutes: "{:.1} minutos",
            seconds: "{} segundos e {.3} ms",
        },
    }
}
