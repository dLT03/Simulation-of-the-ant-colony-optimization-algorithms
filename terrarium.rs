use ggez::graphics::{DrawMode, DrawParam, Rect, Text, TextFragment};
use ggez::graphics::Mesh;
use crate::config::*;
use crate::ants::Ant;
use ggez::graphics::*;
use ggez::{Context, GameError, GameResult};
use ggez::event::MouseButton;
use ggez::input::keyboard::{KeyCode, KeyInput};
use rand::Rng;
use crate::functions::*;
use ggez::mint::Point2;

pub struct Terrarium {

    // Technical
    pre_rendered_tunnel: Mesh,
    state: GameState,
    nest: Position,
    food_amount: i32,

    // Entities
    ants: Vec<Ant>,
    pub pheromones: Vec<Vec<f32>>,
    pub tunnels: Vec<Vec<bool>>,
    pub food_sources: Vec<FoodSource>,
}

impl Terrarium {

    // Initialization of the terrarium
    pub fn new(ctx: &Context) -> Terrarium {

        // Boring technical stuff
        let mut rng = rand::thread_rng();
        let nest_pos = Position { x: (WINDOW_WIDTH / 2.0) as i32, y: (WINDOW_HEIGHT / 2.0) as i32 };
        let pre_rendered_tunnel = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect::new(0.0, 0.0, WINDOW_TO_GAME_SCALE, WINDOW_TO_GAME_SCALE), Color::new(0.4, 0.2, 0.1, 1.0)).unwrap();

        // Generate random food sources
        let mut food_sources = Vec::new();
        for _ in 0..FOOD_SOURCES_COUNT {

            // Until valid coordinates has been found
            loop {
                let food_x = rng.gen_range((FOOD_SCALE/WINDOW_TO_GAME_SCALE) as usize..WIDTH- (FOOD_SCALE/WINDOW_TO_GAME_SCALE) as usize);
                let food_y = rng.gen_range((FOOD_SCALE/WINDOW_TO_GAME_SCALE) as usize..HEIGHT-(FOOD_SCALE/WINDOW_TO_GAME_SCALE) as usize);

                let dx = (nest_pos.get_x_grid() - food_x as i32).abs();
                let dy = (nest_pos.get_y_grid() - food_y as i32).abs();

                if dx + dy > FOOD_DISTANCE {
                    let position = Position {
                        x: (food_x * WINDOW_TO_GAME_SCALE as usize) as i32,
                        y: (food_y * WINDOW_TO_GAME_SCALE as usize) as i32,
                    };
                    food_sources.push(FoodSource { position, amount: FOOD_AMOUNT_PER_SOURCE, });
                    break;
                }
            }
        }

        // Generate ants
        let mut ants = Vec::new();
        for _ in 0..ANT_COUNT { ants.push(Ant::new(&nest_pos)); }

        // Return object terrarium
        Terrarium {
            pre_rendered_tunnel,
            state: GameState::Playing,
            nest: nest_pos,
            food_amount: 0,

            ants,
            pheromones: vec![vec![MIN_PHEROMONES; 2 * HEIGHT]; WIDTH],
            tunnels: vec![vec![false; HEIGHT]; WIDTH],
            food_sources,
        }
    }

    // Gets neighbours of given ant
    pub fn get_all_neighbors(&self, position: Position) -> Vec<Position> {

        // Vector for all neighbours
        let mut neighbors = Vec::new();

        // Potential moves
        let potential_moves = [
            (position.get_x_grid() - 1, position.get_y_grid()), // Up
            (position.get_x_grid() + 1, position.get_y_grid()), // Down
            (position.get_x_grid(), position.get_y_grid() - 1), // Left
            (position.get_x_grid(), position.get_y_grid() + 1), // Right
        ];

        // Add only moves that are in the window
        for (nx, ny) in potential_moves {
            if nx >= 0 && ny >= 0 && (nx as usize) < WIDTH && (ny as usize) < HEIGHT {
                neighbors.push(Position { x: nx * WINDOW_TO_GAME_SCALE as i32, y: ny * WINDOW_TO_GAME_SCALE as i32 });
            }
        }

        neighbors
    }

    // Return food source only if food is nearby
    pub fn scan_for_food(&mut self, pos: Position) -> Option<&mut FoodSource> {
        for food in self.food_sources.iter_mut() {
            let dx = (food.position.get_x_grid() - pos.get_x_grid()).abs();
            let dy = (food.position.get_y_grid() - pos.get_y_grid()).abs();

            if dx + dy <= FOOD_DETECTION_RANGE && food.amount > 0 {
                return Some(food)
            }
        }

        None
    }

    // Checks if there is nest nearby
    pub fn scan_for_nest(&self, pos: Position) -> bool {

        let dx = (self.nest.x - pos.x).abs();
        let dy = (self.nest.y - pos.y).abs();

        if dx + dy <= NEST_DETECTION_RANGE { true }
        else { false }
    }

    // Return bool whether position is tunnel
    pub fn is_tunnel(&self, position: Position) -> bool {
        self.tunnels[position.get_x_grid() as usize][position.get_y_grid() as usize]
    }

    // Spawns food in given position
    fn spawn_food(&mut self, pos: Position) {

        let (x_grid, y_grid) = (pos.get_x_grid(), pos.get_y_grid());

        let dx = (self.nest.get_x_grid() - x_grid).abs();
        let dy = (self.nest.get_y_grid() - y_grid).abs();

        if dx + dy > FOOD_DISTANCE {
            self.food_sources.push(FoodSource { position: pos, amount: FOOD_AMOUNT_PER_SOURCE, });
        }
    }

    // Adds food to the nest
    pub fn add_food(&mut self) {
        self.food_amount+=1;
    }
}

// Update and Draw functions called from game engine
impl ggez::event::EventHandler for Terrarium {

    // Updates every element of game
    fn update(&mut self, _: &mut Context) -> GameResult {
        match self.state {
            GameState::Playing => {

                // Deletes dead food sources
                self.food_sources.retain(|food| food.amount > 0);

                // Update pheromones
                for row in &mut self.pheromones {
                    for pheromone in row.iter_mut() {
                        if *pheromone > MAX_PHEROMONES / 2.0 {
                            *pheromone *= EVAPORATION_RATE_FAST;
                        }
                        else if *pheromone > MIN_PHEROMONES {
                            *pheromone *= EVAPORATION_RATE_SLOW;
                        }
                        else {
                            *pheromone = MIN_PHEROMONES;
                        }
                    }
                }

                // Updates ants - work around borrow checker, function from ChatGPT
                let mut ants = std::mem::take(&mut self.ants);
                for ant in &mut ants { ant.update(self); }

                // Ants go back to terrarium :)
                self.ants = ants;

                Ok(())
            }
            GameState::Paused => {

                // Nothing

                Ok(())
            }
        }
    }

    // After update, this function is called to draw everything
    fn draw(&mut self, ctx: &mut Context) -> GameResult {

        // Create Canvas to draw on
        let mut canvas = Canvas::from_frame(ctx, Color::new(0.22, 0.15, 0.13, 1.0));

        // Render tunnels
        for (x, row) in self.tunnels.iter().enumerate() {
            for (y, &active) in row.iter().enumerate() {
                if active {
                    let position = [(x as f32) * WINDOW_TO_GAME_SCALE, (y as f32) * WINDOW_TO_GAME_SCALE];
                    canvas.draw(&self.pre_rendered_tunnel, DrawParam::default().dest(position));
                }
            }
        }

        // Render pheromones above visibility limit
        for (x, row) in self.pheromones.iter_mut().enumerate() {
            for (y, pheromone) in row.iter().enumerate() {
                if *pheromone > MIN_PHEROMONES {
                    if let Some((mut x1, mut y1, mut x2, mut y2)) = pheromones_to_board(x as i32, y as i32) {
                        // Upscale values
                        x1 *= WINDOW_TO_GAME_SCALE as i32;
                        y1 *= WINDOW_TO_GAME_SCALE as i32;
                        x2 *= WINDOW_TO_GAME_SCALE as i32;
                        y2 *= WINDOW_TO_GAME_SCALE as i32;

                        // Average point to render in half of 2 pieces of tunnel
                        let (px, py) = ((x1 + x2) / 2, (y1 + y2) / 2);

                        // Render
                        let pheromone_mesh = predefined_rectangle_mesh(ctx, PHEROMONE_SIZE, Color::new(1.0, 1.0, 1.0, *pheromone/MAX_PHEROMONES))?;
                        canvas.draw(&pheromone_mesh, DrawParam::default().dest([px as f32, py as f32]));
                    }
                }
            }
        }

        // Render ants
        for ant in &self.ants {
            ant.draw(ctx, &mut canvas)?;
        }

        // Render nest
        let nest_square = predefined_rectangle_mesh(ctx, NEST_SIZE, Color::new(0.141, 0.090, 0.078, 1.0))?;
        let nest_pos = [self.nest.x-(NEST_SIZE/2.0) as i32  , self.nest.y-(NEST_SIZE/2.0) as i32];
        canvas.draw(&nest_square, DrawParam::default().dest([nest_pos[0] as f32, nest_pos[1] as f32]));
        // Render text on nest
        let text = Text::new(TextFragment {
            text: format!("{}", self.food_amount),
            color: Some(Color::WHITE),
            scale: Some(PxScale::from(15.0)),
            ..Default::default()
        });
        let dimensions = text.dimensions(ctx);
        let text_pos = Point2 {
            x: (nest_pos[0] as f32 + NEST_SIZE / 2.0 - dimensions.unwrap().w / 2.0).round(),
            y: (nest_pos[1] as f32 + NEST_SIZE / 2.0 - dimensions.unwrap().h/2.0).round(),
        };
        canvas.draw(&text, DrawParam::default().dest(text_pos));

        // Render food sources
        for food in &self.food_sources {
            if food.amount > 0 {
                let food_size = FOOD_SCALE;
                let pos = [food.position.x-(FOOD_SCALE/2.0) as i32, food.position.y-(FOOD_SCALE/2.0) as i32];
                let square = predefined_rectangle_mesh(ctx, food_size, Color::GREEN)?;
                canvas.draw(&square, DrawParam::default().dest([pos[0] as f32, pos[1] as f32]), );

                // Render text on food
                let text = Text::new(TextFragment {
                    text: format!("{}", food.amount),
                    color: Some(Color::BLACK),
                    scale: Some(PxScale::from(25.0)),
                    ..Default::default()
                });
                let dimensions = text.dimensions(ctx);
                let text_pos = Point2 {
                    x: (pos[0] as f32 + food_size / 2.0 - dimensions.unwrap().w / 2.0).round(),
                    y: (pos[1] as f32 + food_size / 2.0 - dimensions.unwrap().h/2.0).round(),
                };
                canvas.draw(&text, DrawParam::default().dest(text_pos));
            }
        }

        //Finish drawing
        canvas.finish(ctx)?;
        Ok(())
    }

    // Adding food by click
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) -> Result<(), GameError> {
        match button {
            MouseButton::Left => Ok(self.spawn_food(Position { x: x as i32, y: y as i32})),
            _ => Ok(())
        }
    }

    // Play/Pause handler - function from ChatGPT
    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeated: bool) -> Result<(), GameError> {
        if input.keycode == Some(KeyCode::Space) {
            self.state = match self.state {
                GameState::Playing => GameState::Paused,
                GameState::Paused => GameState::Playing,
            };
        }
        Ok(())
    }
}
