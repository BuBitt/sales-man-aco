use bevy::prelude::*;
use rand::prelude::*;
use std::collections::HashSet;

/// Grid-based point distribution to ensure points aren't too close together
pub fn generate_grid_points(count: usize, width: f32, height: f32) -> Vec<Vec2> {
    let mut rng = thread_rng();
    let mut points = Vec::with_capacity(count);
    
    // Calculate grid dimensions based on count
    let grid_size = (count as f32).sqrt().ceil() as usize;
    let cell_width = width / grid_size as f32;
    let cell_height = height / grid_size as f32;
    
    // Generate one point per grid cell with some random offset
    let mut cells: Vec<(usize, usize)> = Vec::new();
    for x in 0..grid_size {
        for y in 0..grid_size {
            cells.push((x, y));
        }
    }
    
    // Shuffle the cells
    cells.shuffle(&mut rng);
    
    // Take the first 'count' cells
    for (_, (x, y)) in cells.into_iter().take(count).enumerate() {
        // Calculate base position
        let base_x = -width / 2.0 + x as f32 * cell_width;
        let base_y = -height / 2.0 + y as f32 * cell_height;
        
        // Add a random offset within the cell (70% of cell size to maintain spacing)
        let offset_x = rng.gen_range(0.0..0.7) * cell_width;
        let offset_y = rng.gen_range(0.0..0.7) * cell_height;
        
        points.push(Vec2::new(
            base_x + offset_x,
            base_y + offset_y,
        ));
    }
    
    points
}

/// Blue noise distribution using a simple Poisson disk sampling
pub fn generate_poisson_points(count: usize, width: f32, height: f32) -> Vec<Vec2> {
    let mut rng = thread_rng();
    let mut points = Vec::with_capacity(count);
    
    // Determine minimum distance between points
    let min_dist = (width * height / (count as f32 * 3.0)).sqrt();
    
    // Try to place points with minimum distance
    let max_attempts = 100; // Limit attempts to avoid infinite loops
    let mut attempts = 0;
    
    // Set of indices in the grid for quick neighbor checking
    let mut occupied = HashSet::new();
    
    while points.len() < count && attempts < max_attempts * count {
        // Generate a random point
        let x = rng.gen_range(-width/2.0..width/2.0);
        let y = rng.gen_range(-height/2.0..height/2.0);
        let point = Vec2::new(x, y);
        
        // Check if it's too close to existing points
        let cell_x = ((x + width/2.0) / min_dist).floor() as i32;
        let cell_y = ((y + height/2.0) / min_dist).floor() as i32;
        let grid_key = (cell_x, cell_y);
        
        let mut too_close = false;
        // Check neighbors in a 3×3 grid around the current cell
        for dx in -2..=2 {
            for dy in -2..=2 {
                let neighbor_key = (cell_x + dx, cell_y + dy);
                if occupied.contains(&neighbor_key) {
                    // Need to check if any point in this cell is too close
                    for existing_point in &points {
                        if point.distance(*existing_point) < min_dist {
                            too_close = true;
                            break;
                        }
                    }
                }
                if too_close {
                    break;
                }
            }
            if too_close {
                break;
            }
        }
        
        if !too_close {
            occupied.insert(grid_key);
            points.push(point);
        }
        
        attempts += 1;
    }
    
    // If we didn't get enough points, fall back to grid-based distribution
    if points.len() < count {
        return generate_grid_points(count, width, height);
    }
    
    points
}
