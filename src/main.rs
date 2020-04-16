mod app;
mod render;
mod ui;

use crate::app::App;

use winit::event_loop::EventLoop;

use anyhow::Result;

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    let app = App::new(window)?;
    app.run(event_loop);
    Ok(())
}
