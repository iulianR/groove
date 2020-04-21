use anyhow::Result;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use crate::ui::DebugUi;

use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::render::Renderer;

pub struct App {
    window: Window,
    pub renderer: Renderer,
    debug_ui: DebugUi,
}

impl App {
    pub fn new(window: Window) -> Result<App> {
        let mut renderer = Renderer::new(&window)?;
        let debug_ui = DebugUi::new(&window, &mut renderer);

        Ok(Self {
            window,
            renderer,
            debug_ui,
        })
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        let mut compiler = shaderc::Compiler::new().expect("could not find shaderc");
        let vs = include_str!("shader.vert");
        let vertex_bin = compiler
            .compile_into_spirv(
                vs,
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();

        let fs = include_str!("shader.frag");
        let fragment_bin = compiler
            .compile_into_spirv(
                fs,
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap();
        
        let vs_module = self.renderer.device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&vertex_bin.as_binary_u8()[..])).unwrap(),
        );

        let fs_module = self.renderer.device.create_shader_module(
            &wgpu::read_spirv(std::io::Cursor::new(&fragment_bin.as_binary_u8()[..])).unwrap(),
        );

        let bind_group_layout =
            self.renderer
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    bindings: &[],
                    label: None,
                });
        let bind_group = self
            .renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                bindings: &[],
                label: None,
            });
        let pipeline_layout =
            self.renderer
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&bind_group_layout],
                });

        let render_pipeline =
            self.renderer
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    layout: &pipeline_layout,
                    vertex_stage: wgpu::ProgrammableStageDescriptor {
                        module: &vs_module,
                        entry_point: "main",
                    },
                    fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                        module: &fs_module,
                        entry_point: "main",
                    }),
                    rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: wgpu::CullMode::None,
                        depth_bias: 0,
                        depth_bias_slope_scale: 0.0,
                        depth_bias_clamp: 0.0,
                    }),
                    primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                    color_states: &[wgpu::ColorStateDescriptor {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        color_blend: wgpu::BlendDescriptor::REPLACE,
                        alpha_blend: wgpu::BlendDescriptor::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                    depth_stencil_state: None,
                    vertex_state: wgpu::VertexStateDescriptor {
                        index_format: wgpu::IndexFormat::Uint16,
                        vertex_buffers: &[],
                    },
                    sample_count: 1,
                    sample_mask: !0,
                    alpha_to_coverage_enabled: false,
                });

        let mut hidpi_factor = 1.0;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
                    ..
                } => {
                    hidpi_factor = scale_factor;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    let size = self.window.inner_size();
                    self.renderer.resize(size.width, size.height, hidpi_factor);
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        },
                    ..
                }
                | Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                Event::NewEvents(_) => self.debug_ui.update(),
                Event::MainEventsCleared => {
                    self.debug_ui.prepare_frame(&self.window);
                    self.window.request_redraw()
                }
                Event::RedrawRequested(_) => {
                    let frame = self
                        .renderer
                        .swap_chain()
                        .get_next_texture()
                        .expect("Timeout when acquiring next swap chain texture");

                    let mut encoder = self
                        .renderer
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    {
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: &frame.view,
                                resolve_target: None,
                                load_op: wgpu::LoadOp::Clear,
                                store_op: wgpu::StoreOp::Store,
                                clear_color: wgpu::Color::GREEN,
                            }],
                            depth_stencil_attachment: None,
                        });
                        rpass.set_pipeline(&render_pipeline);
                        rpass.set_bind_group(0, &bind_group, &[]);
                        rpass.draw(0..3, 0..1);
                    }

                    self.debug_ui
                        .draw(&mut encoder, &self.renderer, &frame, &self.window);

                    self.renderer.queue.submit(&[encoder.finish()]);
                }
                Event::RedrawEventsCleared => {
                    // let delta_s = last_frame.elapsed();
                    // last_frame = imgui.io_mut().update_delta_time(last_frame);
                    //
                }
                _ => {}
            }

            self.debug_ui.handle_event(&event, &self.window)
        });
    }
}
