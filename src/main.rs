mod app;
mod gpu;

use app::App;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("run_app");
}
