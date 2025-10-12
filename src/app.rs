use crate::gpu::GpuState;
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

pub struct App {
    // We store a leaked 'static reference to the Window so the GPU surface can also be 'static.
    window: Option<&'static Window>,
    gpu: Option<GpuState>,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            gpu: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window once the app is resumed.
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title("wgpu hello (Triangle)")
                .with_visible(true);
            let window = event_loop.create_window(attrs).expect("window");
            // Leak the window to obtain a &'static Window for this demo.
            let window_static: &'static Window = Box::leak(Box::new(window));

            // Init GPU state synchronously using pollster
            let state = pollster::block_on(GpuState::new(window_static));

            self.gpu = Some(state);
            self.window = Some(window_static);
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let (Some(window), Some(gpu)) = (self.window.as_ref(), self.gpu.as_mut()) {
            if window.id() != window_id {
                return;
            }
            match event {
                WindowEvent::CloseRequested => {
                    // Quit the app
                    std::process::exit(0);
                }
                WindowEvent::Resized(new_size) => {
                    gpu.resize(new_size);
                }
                WindowEvent::RedrawRequested => {
                    match gpu.render() {
                        Ok(()) => {}
                        // On macOS, SurfaceError::Outdated can occur after display changes; just reconfigure.
                        Err(SurfaceError::Outdated) | Err(SurfaceError::Lost) => {
                            gpu.resize(gpu.size);
                        }
                        Err(SurfaceError::OutOfMemory) => {
                            eprintln!("Out of memory");
                            std::process::exit(1);
                        }
                        Err(e) => eprintln!("render error: {e:?}"),
                    }
                    window.request_redraw(); // continuous redraw (simple for demo)
                }
                // Other events intentionally ignored for this minimal example.
                _ => {}
            }
        }
    }
}
