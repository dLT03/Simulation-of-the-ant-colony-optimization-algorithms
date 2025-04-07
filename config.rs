// VISUAL/TECHNICAL
pub const WIDTH: usize = 180;
pub const HEIGHT: usize = 120;
pub const WINDOW_WIDTH: f32 = 900.0;
pub const WINDOW_HEIGHT: f32 = 600.0;
pub const WINDOW_TO_GAME_SCALE: f32 = WINDOW_WIDTH / WIDTH as f32;
pub const ANT_SCALE: f32 = 3.0;
pub const FOOD_SCALE: f32 = 30.0;
pub const FOOD_DISTANCE: i32 = 10;
pub const NEST_SIZE: f32 = 30.0;
pub const PHEROMONE_SIZE: f32 = 2.0;



// GAMEPLAY VARIABLES
pub const UNLIMITED_FOOD: bool = false;
pub const ANT_SOIL_LIMIT: i32 = 100;
pub const NEST_DETECTION_RANGE: i32 = 25;
pub const FOOD_DETECTION_RANGE: i32 = 5;
pub const ANT_COUNT: usize = 100;
pub const FOOD_SOURCES_COUNT: usize = 3;
pub const FOOD_AMOUNT_PER_SOURCE: usize = 50;



// ANT AI
pub const PHEROMONES_INTENSITY: f32 = 20000.0;
pub const EVAPORATION_RATE_FAST: f32 = 0.9;
pub const EVAPORATION_RATE_SLOW: f32 = 0.99;
pub const DESIRABILITY_PHEROMONES: f32 = 7.0;
pub const DESIRABILITY_HEURISTICS: f32 = 2.0;
pub const MIN_PHEROMONES: f32 = 1.0;
pub const MAX_PHEROMONES: f32 = 2000.0;
pub const DIGGING_COST: f32 = 100.0;
