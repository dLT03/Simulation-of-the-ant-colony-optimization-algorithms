mod terrarium;
mod config;
mod ants;
mod functions;

use terrarium::*;
use config::*;
use ggez::event::{self};
use ggez::{Context, ContextBuilder, GameResult};

// Build the context and event loop for the game (needed by game engine)
fn build_context() -> GameResult<(Context, event::EventLoop<()>)> {
    ContextBuilder::new("ant_simulation", "Dominika")
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()
}

// Main function
fn main() -> GameResult {
    let (ctx, event_loop) = build_context()?;  // Build context and event loop
    let state = Terrarium::new(&ctx);  // Initialize the game state with a fresh environment
    event::run(ctx, event_loop, state)  // Run the game loop
}
