use anyhow::Result;

use std::borrow::BorrowMut;
use wgpu::{Device, Queue, Surface, SwapChain};
use winit::window::Window;

pub struct Renderer {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
    pub swap_chain: SwapChain,

    width: u32,
    height: u32,
    hidpi_factor: f64,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Self> {
        let surface = Surface::create(window);
        let adapter = futures::executor::block_on(wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        ))
        .unwrap();

        let (device, queue) =
            futures::executor::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            }));

        let size = window.inner_size();
        let swap_chain = new_swap_chain(&surface, size.width, size.height, &device);

        Ok(Renderer {
            device,
            queue,
            surface,
            swap_chain,
            width: size.width,
            height: size.height,
            hidpi_factor: 1.0,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32, hidpi_factor: f64) {
        self.width = width;
        self.height = height;
        self.hidpi_factor = hidpi_factor;
        self.swap_chain = new_swap_chain(&self.surface, width, height, &self.device);
    }

    pub fn swap_chain(&mut self) -> &mut SwapChain {
        self.swap_chain.borrow_mut()
    }
}

fn new_swap_chain(
    surface: &wgpu::Surface,
    width: u32,
    height: u32,
    device: &wgpu::Device,
) -> wgpu::SwapChain {
    device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width,
            height,
            present_mode: wgpu::PresentMode::Mailbox,
        },
    )
}
