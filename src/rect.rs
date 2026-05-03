use glam::Mat4;
use sdl3::{
    Error,
    gpu::{CommandBuffer, CopyPass, RenderPass, ShaderFormat, ShaderStage, VertexElementFormat},
};

use crate::{
    buffer::BufferHelper,
    color::Color,
    pipeline::{
        Pipeline, PipelineManager, instance_buffer, vertex_attribute, vertex_buffer,
        vertex_input_builder,
    },
    window::UIWindow,
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RectVertexData {
    pub pos: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RectInstanceData {
    pub pos: [f32; 2],
    pub dim: [f32; 2],
    pub col: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}

pub struct RectParams {
    pub rect: Rect,
    pub color: Color,
}

impl RectParams {
    pub fn new(rect: Rect, color: Color) -> Self {
        Self { rect, color }
    }
}

pub struct RectPipelineManager {
    initialized: bool,
    pipeline: Option<Pipeline<RectVertexData, RectInstanceData>>,
    data: Vec<RectInstanceData>,
}

impl RectPipelineManager {
    pub fn new() -> Self {
        Self {
            initialized: false,
            pipeline: None,
            data: vec![],
        }
    }

    pub fn draw(&mut self, rect: RectParams) {
        self.data.push(RectInstanceData {
            pos: [rect.rect.x, rect.rect.y],
            dim: [rect.rect.w, rect.rect.h],
            col: rect.color.to_slice(),
        });
    }
}

impl PipelineManager for RectPipelineManager {
    fn init(&mut self, window: &UIWindow) -> Result<(), Error> {
        if self.initialized {
            panic!("Init called when already initialized!")
        }

        let vs_source = include_bytes!(concat!(env!("OUT_DIR"), "/rect.vert.spv"));
        let fs_source = include_bytes!(concat!(env!("OUT_DIR"), "/rect.frag.spv"));

        let vs_shader = window
            .device
            .create_shader()
            .with_code(ShaderFormat::SPIRV, vs_source, ShaderStage::Vertex)
            .with_entrypoint(c"main")
            .with_uniform_buffers(1)
            .build()?;
        let fs_shader = window
            .device
            .create_shader()
            .with_code(ShaderFormat::SPIRV, fs_source, ShaderStage::Fragment)
            .with_entrypoint(c"main")
            .build()?;

        let vertices = vec![
            RectVertexData { pos: [0.0, 0.0] },
            RectVertexData { pos: [1.0, 0.0] },
            RectVertexData { pos: [0.0, 1.0] },
            RectVertexData { pos: [1.0, 0.0] },
            RectVertexData { pos: [1.0, 1.0] },
            RectVertexData { pos: [0.0, 1.0] },
        ];

        let instances = vec![];

        let vertices_buffer = BufferHelper::new(&window.device, vertices, 6)?;
        let instances_buffer = BufferHelper::new(&window.device, instances, 1000)?;

        self.pipeline = Some(Pipeline::new(
            &window.window,
            &window.device,
            &vs_shader,
            vertex_input_builder(
                &[
                    vertex_buffer::<RectVertexData>(0),
                    instance_buffer::<RectInstanceData>(1),
                ],
                &[
                    vertex_attribute(0, 0, 0, VertexElementFormat::Float2),
                    vertex_attribute(1, 1, 0, VertexElementFormat::Float2),
                    vertex_attribute(1, 2, 8, VertexElementFormat::Float2),
                    vertex_attribute(1, 3, 16, VertexElementFormat::Float4),
                ],
            ),
            &fs_shader,
            vertices_buffer,
            instances_buffer,
        )?);

        self.initialized = true;

        Ok(())
    }

    fn copy(&mut self, window: &UIWindow, _: &CommandBuffer, pass: &CopyPass) -> Result<(), Error> {
        if !self.initialized {
            panic!("RectPipelineManager not initialized.");
        }

        let pipeline = self.pipeline.as_mut().unwrap();
        pipeline.use_instance_data(&window.device, &self.data);
        pipeline.copy(pass)?;

        Ok(())
    }

    fn render(
        &self,
        window: &UIWindow,
        command: &CommandBuffer,
        pass: &RenderPass,
    ) -> Result<(), Error> {
        if !self.initialized {
            panic!("RectPipelineManager not initialized.");
        }

        let (width, height) = window.window.size();
        let proj = Mat4::orthographic_rh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);

        command.push_vertex_uniform_data(0, &proj);
        self.pipeline.as_ref().unwrap().draw(pass);

        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        self.data.clear();

        Ok(())
    }
}
