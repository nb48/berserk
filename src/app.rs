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
    vertical_offset: f32,
    vertical_velocity: f32,
    on_ground: bool,
    rotation: f32, // Rotation angle in radians
    rotating_left: bool,
    rotating_right: bool,
    total_rotation: f32, // Track total rotation during jump
    explosion_timer: f32, // Timer for explosion animation (0.0 = no explosion)
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            gpu: None,
            vertical_offset: -0.3, // Start at floor level
            vertical_velocity: 0.0,
            on_ground: true,
            rotation: 0.0,
            rotating_left: false,
            rotating_right: false,
            total_rotation: 0.0,
            explosion_timer: 0.0,
        }
    }

    fn update_physics(&mut self) {
        const GRAVITY: f32 = -0.002;
        const FLOOR_Y: f32 = -0.3;
        const ROTATION_SPEED: f32 = 0.15; // Faster rotation per frame
        const PI: f32 = std::f32::consts::PI;

        // Update explosion timer
        if self.explosion_timer > 0.0 {
            self.explosion_timer -= 0.02;
            if self.explosion_timer <= 0.0 {
                self.explosion_timer = 0.0;
            }
        }

        // Apply gravity
        if !self.on_ground {
            self.vertical_velocity += GRAVITY;

            // Apply rotation while in air
            if self.rotating_left {
                self.rotation -= ROTATION_SPEED;
                self.total_rotation -= ROTATION_SPEED;
            }
            if self.rotating_right {
                self.rotation += ROTATION_SPEED;
                self.total_rotation += ROTATION_SPEED;
            }
        }

        // Update position
        self.vertical_offset += self.vertical_velocity;

        // Check floor collision
        if self.vertical_offset <= FLOOR_Y {
            self.vertical_offset = FLOOR_Y;
            self.vertical_velocity = 0.0;
            self.on_ground = true;

            // Check for successful 360 trick landing
            // Must have rotated at least 360 degrees (2*PI radians)
            // AND land with rotation close to 0 (within a more forgiving tolerance)
            let completed_full_rotation = self.total_rotation.abs() >= 2.0 * PI;

            // Normalize rotation to -PI to PI range for checking
            let normalized_rotation = self.rotation % (2.0 * PI);
            let normalized_rotation = if normalized_rotation > PI {
                normalized_rotation - 2.0 * PI
            } else if normalized_rotation < -PI {
                normalized_rotation + 2.0 * PI
            } else {
                normalized_rotation
            };

            let landed_even = normalized_rotation.abs() < 0.017; // ~1 degree tolerance
            let degrees = normalized_rotation.abs() * 180.0 / PI;

            // Debug output
            if self.total_rotation.abs() > 0.5 {
                println!(
                    "Landing: {:.1}° off | Total rotation: {:.1}° | 360 complete: {}",
                    degrees,
                    self.total_rotation.abs() * 180.0 / PI,
                    completed_full_rotation
                );
            }

            if completed_full_rotation && landed_even {
                // EXPLOSION!
                self.explosion_timer = 1.0; // Start explosion animation
                println!("💥 PERFECT 360 LANDING! 💥");
            }

            self.rotation = 0.0; // Reset rotation when landing
            self.total_rotation = 0.0; // Reset total rotation counter
            self.rotating_left = false; // Clear rotation inputs on landing
            self.rotating_right = false;
        } else {
            self.on_ground = false;
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
        // Check window ID first
        if let Some(window) = self.window.as_ref() {
            if window.id() != window_id {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                // Quit the app
                std::process::exit(0);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                use winit::keyboard::KeyCode;
                let is_pressed = key_state == winit::event::ElementState::Pressed;

                match key_code {
                    KeyCode::ArrowLeft => {
                        // Set rotation state only when in air
                        if !self.on_ground {
                            self.rotating_left = is_pressed;
                        }
                    }
                    KeyCode::ArrowRight => {
                        // Set rotation state only when in air
                        if !self.on_ground {
                            self.rotating_right = is_pressed;
                        }
                    }
                    KeyCode::ArrowUp => {
                        if is_pressed {
                            self.vertical_offset += 0.05;
                            if let Some(window) = self.window {
                                window.request_redraw();
                            }
                        }
                    }
                    KeyCode::ArrowDown => {
                        if is_pressed {
                            self.vertical_offset -= 0.05;
                            if let Some(window) = self.window {
                                window.request_redraw();
                            }
                        }
                    }
                    KeyCode::Space => {
                        // Jump only if on ground and key is pressed
                        if is_pressed && self.on_ground {
                            self.vertical_velocity = 0.08; // Jump velocity
                            self.on_ground = false;
                            if let Some(window) = self.window {
                                window.request_redraw();
                            }
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(new_size);
                }
            }
            WindowEvent::RedrawRequested => {
                // Update physics
                self.update_physics();

                if let Some(gpu) = self.gpu.as_mut() {
                    match gpu.render(self.vertical_offset, self.rotation, self.explosion_timer) {
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
                }

                if let Some(window) = self.window {
                    window.request_redraw(); // continuous redraw (simple for demo)
                }
            }
            // Other events intentionally ignored for this minimal example.
            _ => {}
        }
    }
}
