use crate::render;
use imgui::im_str;
use imgui::{Condition, Context};
use imgui_wgpu::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::borrow::BorrowMut;
use std::time::{Duration, Instant};
use wgpu::{CommandEncoder, SwapChainOutput, TextureFormat};
use winit::event::Event;
use winit::window::Window;

pub struct DebugUi {
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub ui_renderer: Renderer,
    last_frame: Instant,
    delta_s: Duration,
    demo_open: bool,
}

impl DebugUi {
    pub fn new(window: &Window, renderer: &mut render::Renderer) -> DebugUi {
        let mut imgui = Context::create();
        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);
        imgui.set_ini_filename(None);

        let ui_renderer = Renderer::new(
            &mut imgui,
            &renderer.device,
            &mut renderer.queue,
            TextureFormat::Bgra8UnormSrgb,
            None,
        );

        let last_frame = Instant::now();
        DebugUi {
            imgui,
            platform,
            ui_renderer,
            last_frame,
            delta_s: last_frame.elapsed(),
            demo_open: false,
        }
    }

    pub fn draw(
        &mut self,
        encoder: &mut CommandEncoder,
        renderer: &render::Renderer,
        frame: &SwapChainOutput,
        window: &Window,
    ) {
        let ui = self.imgui.frame();
        let delta_s = self.delta_s;

        {
            let window = imgui::Window::new(im_str!("Hello world"));
            window
                .size([300.0, 100.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(im_str!("Hello world!"));
                    ui.text(im_str!("This...is...imgui-rs on WGPU!"));
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(im_str!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0],
                        mouse_pos[1]
                    ));
                });

            let window = imgui::Window::new(im_str!("Hello too"));
            window
                .size([400.0, 200.0], Condition::FirstUseEver)
                .position([400.0, 200.0], Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text(im_str!("Frametime: {:?}", delta_s));
                });

            ui.show_demo_window(&mut self.demo_open);
        }

        self.platform.prepare_render(&ui, &window);

        self.ui_renderer
            .render(
                ui.render(),
                &renderer.device,
                &mut encoder.borrow_mut(),
                &frame.view,
            )
            .expect("rendering failed");
    }

    pub fn update(&mut self) {
        self.delta_s = self.last_frame.elapsed();
        self.last_frame = self.imgui.io_mut().update_delta_time(self.last_frame);
    }

    pub fn prepare_frame(&mut self, window: &Window) {
        self.platform
            .prepare_frame(self.imgui.io_mut(), &window)
            .expect("Failed to prepare frame");
    }

    pub fn handle_event(&mut self, event: &Event<()>, window: &Window) {
        self.platform
            .handle_event(self.imgui.io_mut(), &window, &event);
    }
}
