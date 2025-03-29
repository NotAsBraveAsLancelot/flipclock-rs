mod clock;
mod config;
mod graphics_engine;
mod graphics_engine_impl;
use clock::FlipClock;
use config::Config;
use graphics_engine_impl::Sdl2GraphicsEngine;

fn main() -> Result<(), String> {
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let config = Config::load().map_err(|e| e.to_string())?;
    let settings = config.to_clock_settings();
    let engine = Sdl2GraphicsEngine::new(&ttf_context, &settings)?;

    let mut clock = FlipClock::new(engine, &settings);
    clock.run()
}
