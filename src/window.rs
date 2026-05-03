use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use sdl3::{
    VideoSubsystem,
    gpu::{ColorTargetInfo, Device, ShaderFormat},
    pixels::Color,
    sys::gpu::{SDL_GPULoadOp, SDL_GPUStoreOp},
    video::Window,
};

use crate::pipeline::PipelineManager;

pub struct UIWindow {
    pub window: Window,
    pub device: Device,
    pipelines: Vec<Rc<RefCell<dyn PipelineManager>>>,
}

impl UIWindow {
    pub fn new(
        video: &VideoSubsystem,
        title: &str,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let window = video
            .window(title, width, height)
            .position_centered()
            .resizable()
            .opengl()
            .build()?;

        let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&window)?;

        Ok(Self {
            window,
            device,
            pipelines: vec![],
        })
    }

    pub fn add_pipeline(
        &mut self,
        pipeline: Rc<RefCell<dyn PipelineManager>>,
    ) -> Result<(), sdl3::Error> {
        pipeline.borrow_mut().init(self)?;
        self.pipelines.push(pipeline);
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), sdl3::Error> {
        let mut command_buffer = self.device.acquire_command_buffer()?;

        let copy_pass = self.device.begin_copy_pass(&command_buffer)?;

        for pipeline in &self.pipelines {
            pipeline
                .borrow_mut()
                .copy(self, &command_buffer, &copy_pass)?;
        }

        self.device.end_copy_pass(copy_pass);

        if let Ok(swapchain) = command_buffer.wait_and_acquire_swapchain_texture(&self.window) {
            let color_targets = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_clear_color(Color::RGB(0, 0, 0))
                .with_load_op(SDL_GPULoadOp::CLEAR)
                .with_store_op(SDL_GPUStoreOp::STORE)];

            let render_pass =
                self.device
                    .begin_render_pass(&command_buffer, &color_targets, None)?;

            for pipeline in &self.pipelines {
                pipeline
                    .borrow()
                    .render(self, &command_buffer, &render_pass)?;
            }

            self.device.end_render_pass(render_pass);
            command_buffer.submit()?;
        } else {
            command_buffer.cancel();
        }

        for pipeline in &mut self.pipelines {
            pipeline.borrow_mut().finish()?;
        }

        Ok(())
    }
}
