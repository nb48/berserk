use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct GpuState {
    // Surface borrows the Instance; we create the Instance as a leaked 'static to satisfy
    // the surface lifetime for this simple demo.
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl GpuState {
    pub async fn new(window: &'static Window) -> Self {
        // Instance
        // Leak the Instance to get a 'static reference so Surface<'static> is usable in the
        // returned struct. This is acceptable for a short-lived demo program.
        let instance: &'static wgpu::Instance = Box::leak(Box::new(wgpu::Instance::default()));
        // Create the surface from the window handle. No unsafe block is required here.
        let surface = instance.create_surface(window).expect("create surface");

        // Choose an adapter that can present to this surface
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("No suitable GPU adapters found");

        // Device + queue
        // Request device/queue from the adapter. Use the adapter-provided limits.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                // other fields use defaults via struct update
                ..Default::default()
            })
            .await
            .expect("request_device failed");

        // Surface configuration
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            // desired_maximum_frame_latency is newer; set default via Default::default if not present
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/triangle.wgsl").into()),
        });

        // Create uniform buffer for the vertical offset
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform buffer"),
            size: 16, // vec4<f32> alignment (we only use one f32 but need alignment)
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Pipeline (no vertex buffer; we use builtin vertex_index)
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[], // no vertex buffer
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            uniform_buffer,
            bind_group,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, vertical_offset: f32, rotation: f32, explosion_timer: f32) -> Result<(), SurfaceError> {
        // Update the uniform buffer with the new offset, rotation, and explosion timer
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[vertical_offset, rotation, explosion_timer, 0.0f32]),
        );

        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.07,
                            b: 0.10,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.draw(0..3, 0..1); // Triangle
            rpass.draw(3..9, 0..1); // Floor (6 vertices for 2 triangles)
            rpass.draw(9..39, 0..1); // Explosion particles (30 vertices for 10 triangles)
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}
