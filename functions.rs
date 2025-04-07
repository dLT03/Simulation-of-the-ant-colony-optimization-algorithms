use ggez::{Context, GameResult};
use crate::config::*;

// Translates location of two tunnels to location of pheromone linking these tunnel
pub fn board_to_pheromones(x1: i32, y1: i32, x2: i32, y2: i32) -> Option<(i32, i32)> {
    if x1 == x2 {
        return Some((x1, y1 + y2))
    }
    if y1 == y2 {
        return Some((x1.min(x2), 2 * y1))
    }
    None
}

// Translates location of pheromone linking tunnels to location of these tunnels
pub fn pheromones_to_board(x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {

    // Horizontal neighbours
    if y%2==0 {

        // Bad location given
        if x==0 {
            return None
        }

        // Valid location given
        return Some((x-1, y/2, x, y/2))
    }

    // Vertical neighbours
    Some((x,y/2,x,(y/2)+1))
}


// Helper function for predefined rectangle
pub fn predefined_rectangle_mesh(ctx: &mut Context, size: f32, color: ggez::graphics::Color) -> GameResult<ggez::graphics::Mesh> {
    ggez::graphics::Mesh::new_rectangle(ctx, ggez::graphics::DrawMode::fill(), ggez::graphics::Rect::new(0.0, 0.0, size, size), color)
}


// STRUCTURES & ENUMS
#[derive(Clone)]
pub struct FoodSource {
    pub position: Position,
    pub amount: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

 // To get the map coordinates
impl Position{
    pub fn get_x_grid(&self) -> i32 {
        self.x / WINDOW_TO_GAME_SCALE as i32
    }

    pub fn get_y_grid(&self) -> i32 {
        self.y / WINDOW_TO_GAME_SCALE as i32
    }
}

pub enum GameState {
    Playing,
    Paused,
}