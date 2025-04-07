use ggez::{Context, GameResult};
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, Mesh};
use ggez::mint::Point2;
use crate::config::*;
use crate::functions::*;
use crate::terrarium::Terrarium;
use std::collections::HashMap;
use rand::Rng;

pub struct Ant {

    pub position: Position,
    nest_position: Position,
    path_positions: Vec<Position>,
    pub returning: bool, // Ant either digs tunnel or returns
    carrying_food: bool, // Ant can return with or without food
    soil_carried: i32,  // Progress toward carrying soil back to nest
    visited_fields: Vec<Vec<bool>>,
    direction: f64,
}

impl Ant {

    // Creates new Ant
    pub(crate) fn new(pos: &Position) -> Self {

        // Randomise staring position and direction
        let mut rng = rand::thread_rng();
        let random_direction = rng.gen_range(0.0..360.0);
        let rand_offset_x = rng.gen_range(-5.0..=5.0);
        let rand_offset_y = rng.gen_range(-5.0..=5.0);
        let starting_position = Position { x: pos.x + rand_offset_x as i32, y: pos.y + rand_offset_y as i32, };

        Ant {
            position: starting_position,
            nest_position: pos.clone(),
            path_positions: Vec::new(),
            returning: false,
            carrying_food: false,
            soil_carried: 0,
            visited_fields: vec![vec![false; HEIGHT]; WIDTH],
            direction: random_direction,
        }

    }

    // Behaviour of ant
    pub fn update(&mut self, terrarium: &mut Terrarium) {

        // Ant is just full of soil :(
        if self.returning && !self.carrying_food {
            self.go_back_one_move();
            self.scan_for_target(terrarium);
            return;
        }

        // Starting position of ant
        if self.path_positions.is_empty() {
            let offset_x = self.direction.to_radians().cos();
            let offset_y = self.direction.to_radians().sin();
            self.position.x += offset_x as i32;
            self.position.y += offset_y as i32;

            self.mark_visited();
            self.add_path();
            return;
        }

        // Find neighbours
        let neighbors: Vec<Position> = self.find_neighbours(terrarium)
            .into_iter()
            .filter(|&neighbour| { if self.carrying_food { terrarium.is_tunnel(neighbour) } else { true } })
            .collect();

        if neighbors.is_empty(){
            self.go_back_one_move();
            return;
        }

        // Find best move
        let probabilities = self.calculate_probabilities(terrarium, &neighbors);
        if let Some(new_position) = self.select_next_position(probabilities) {
            self.move_and_dig(terrarium, new_position);
            self.mark_visited();
            self.add_path();
        }

        // Evaluate move
        self.scan_for_target(terrarium);
        self.check_if_full();
    }

    // Draw ant
    pub(crate) fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {

        let point = Point2 { x: self.position.x as f32, y: self.position.y as f32, };

        let ant_hungry = Mesh::new_circle(ctx, DrawMode::fill(), point, ANT_SCALE, 0.1, Color::BLACK, )?;
        // Ant carrying food
        let ant_happy = Mesh::new_circle(ctx, DrawMode::fill(), point, ANT_SCALE, 0.1, Color::GREEN, )?;
        // Ant carrying soil to the nest
        let ant_heavy = Mesh::new_circle(ctx, DrawMode::fill(), point, ANT_SCALE, 0.1, Color::new(0.561, 0.361, 0.231, 1.0))?;

        if self.carrying_food {
            canvas.draw(&ant_happy, DrawParam::default());
        }
        else if self.soil_carried == ANT_SOIL_LIMIT {
                canvas.draw(&ant_heavy, DrawParam::default());
            }
        else {
            canvas.draw(&ant_hungry, DrawParam::default());
        }

        Ok(())
    }

    // Returns to nest from the memorised position
    fn go_back_one_move(&mut self) {
        while let Some(previous_move) = self.path_positions.pop() {

            if self.position != previous_move {
                self.position = previous_move;
                return;
            }
        }
    }

    // Marks field as visited so ant won't come back here
    fn mark_visited(&mut self) {

        // Gets grid coords
        let (grid_x, grid_y) = (self.position.get_x_grid(), self.position.get_y_grid());

        // If field is valid, set it to visited
        if grid_x >= 0 && grid_x < WIDTH as i32 && grid_y >= 0 && grid_y < HEIGHT as i32 {
            self.visited_fields[grid_x as usize][grid_y as usize] = true;
        }
    }

    // Adds current field to memory
    fn add_path(&mut self) {
        self.path_positions.push(self.position);
    }

    // Finds non-visited neighbours
    fn find_neighbours(&self, terrarium: &mut Terrarium) -> Vec<Position> {
        let all_neighbours = terrarium.get_all_neighbors(self.position);
        let mut valid_neighbours = Vec::new();

        for neighbour in all_neighbours {
            let (nx, ny) = (neighbour.get_x_grid(), neighbour.get_y_grid());

            if nx >= 0 && nx < WIDTH as i32 && ny >= 0 && ny < HEIGHT as i32 &&
                !self.visited_fields[nx as usize][ny as usize] {
                valid_neighbours.push(neighbour);
            }
        }
        valid_neighbours
    }

    // Returns heuristics value
    fn heuristics(terrarium: &Terrarium, pos: &Position) -> f32 {
        if terrarium.is_tunnel(*pos) {
            1.0
        } else {
            1.0 / DIGGING_COST
        }
    }

    // Calculates probability for every path
    fn calculate_probabilities(&self, terrarium: &Terrarium, neighbors: &Vec<Position>) -> HashMap<Position, f32> {
        let mut desirabilities = HashMap::new();
        let mut total_desire = 0.0;

        for neighbor in neighbors {
            if let Some((px, py)) = board_to_pheromones(self.position.get_x_grid(), self.position.get_y_grid(), neighbor.get_x_grid(), neighbor.get_y_grid()) {
                let pheromone = terrarium.pheromones[px as usize][py as usize];
                let heuristic = Ant::heuristics(terrarium, neighbor);
                let desirability = pheromone.powf(DESIRABILITY_PHEROMONES) * heuristic.powf(DESIRABILITY_HEURISTICS);
                desirabilities.insert(neighbor, desirability);
                total_desire += desirability;
            }
        }

        // Pack neighbours and desirabilities into one hashMap
        let mut probabilities = HashMap::new();
        for (&neighbor, &desirability) in &desirabilities {
            probabilities.insert(*neighbor, desirability / total_desire);
        }

        probabilities
    }

    // Select next move probabilistically
    fn select_next_position(&mut self, probabilities: HashMap<Position, f32>) -> Option<Position> {
        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen(); // [0, 1]
        let mut cumulative_probability = 0.0;

        // Apply random value to hashmap
        for (position, probability) in probabilities {
            cumulative_probability += probability;
            if random_value <= cumulative_probability.into() {
                return Some(position)
            }
        }

        None
    }

    // Moves ant by its velocity, doesn't think whether it makes sense
    fn move_and_dig(&mut self, terrarium: &mut Terrarium, pos: Position) {

        // Moves
        self.position = pos;

        let (grid_x , grid_y) = (self.position.get_x_grid(), self.position.get_y_grid());

        // Digs tunnel
        if !terrarium.tunnels[grid_x as usize][grid_y as usize] {
            terrarium.tunnels[grid_x as usize][grid_y as usize] = true;
            self.soil_carried += 1;
        }
    }

    // Looks for nest or food depends on context
    fn scan_for_target(&mut self, terrarium: &mut Terrarium) {
        if let Some(food) = terrarium.scan_for_food(self.position) {
            if !self.returning {
                self.found_food(food);
            }
        }

        if terrarium.scan_for_nest(self.position) {
            if self.returning {
                self.found_nest(terrarium);
            }
        }
    }

    // Check if ant should be returning
    fn check_if_full(&mut self) {
        if self.soil_carried >= ANT_SOIL_LIMIT && !self.returning {
            self.returning = true;
        }
    }

    // Sets ant to go back to nest and release pheromones
    fn found_food(&mut self, food_source: &mut FoodSource) {
        if UNLIMITED_FOOD == false {
                food_source.amount -= 1;
        }
        self.returning = true;
        self.carrying_food = true;
        self.visited_fields = vec![vec![false; HEIGHT]; WIDTH];
        self.path_positions.clear();
    }

    // Resets ant to factory settings, spreads pheromones if necessary
    fn found_nest(&mut self, terrarium: &mut Terrarium) {

        // Spreads pheromones if found food
        if self.carrying_food {
            self.spread_pheromones(terrarium);
            self.carrying_food = false;
            terrarium.add_food();
        }

        // Clears everything else
        self.position = self.nest_position;
        self.returning = false;
        self.soil_carried = 0;
        self.visited_fields = vec![vec![false; HEIGHT]; WIDTH];
        self.path_positions.clear();
    }

    // Spreads pheromones at memorised locations (food -> nest)
    fn spread_pheromones(&mut self, terrarium: &mut Terrarium) {

        let spread_value = PHEROMONES_INTENSITY / self.path_positions.len() as f32;
        while let Some(pos_1) = self.path_positions.pop() {
            if let Some(pos_2) = self.path_positions.last() {
                if let Some((px, py)) = board_to_pheromones(pos_1.get_x_grid(), pos_1.get_y_grid(), pos_2.get_x_grid(), pos_2.get_y_grid()) {
                    terrarium.pheromones[px as usize][py as usize] += spread_value;

                    if terrarium.pheromones[px as usize][py as usize] > MAX_PHEROMONES {
                        terrarium.pheromones[px as usize][py as usize] = MAX_PHEROMONES;
                    }
                }
            }
        }
    }
}