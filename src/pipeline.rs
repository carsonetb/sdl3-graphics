use sdl3::{
    Error,
    gpu::{
        BlendFactor, BlendOp, ColorComponentFlags, ColorTargetBlendState, ColorTargetDescription,
        CommandBuffer, CopyPass, CullMode, Device, GraphicsPipeline, GraphicsPipelineTargetInfo,
        PrimitiveType, RasterizerState, RenderPass, Shader, VertexAttribute,
        VertexBufferDescription, VertexElementFormat, VertexInputRate, VertexInputState,
    },
    video::Window,
};

use crate::{buffer::BufferHelper, window::UIWindow};

pub fn vertex_buffer<T>(slot: u32) -> VertexBufferDescription {
    println!("{}", size_of::<T>());
    VertexBufferDescription::default()
        .with_slot(slot)
        .with_input_rate(VertexInputRate::Vertex)
        .with_pitch(size_of::<T>() as u32)
}

pub fn instance_buffer<T>(slot: u32) -> VertexBufferDescription {
    VertexBufferDescription::default()
        .with_slot(slot)
        .with_input_rate(VertexInputRate::Instance)
        .with_pitch(size_of::<T>() as u32)
}

pub fn vertex_attribute(
    slot: u32,
    location: u32,
    offset: u32,
    format: VertexElementFormat,
) -> VertexAttribute {
    VertexAttribute::default()
        .with_buffer_slot(slot)
        .with_location(location)
        .with_offset(offset)
        .with_format(format)
}

pub fn vertex_input_builder(
    buffers: &[VertexBufferDescription],
    attributes: &[VertexAttribute],
) -> VertexInputState {
    VertexInputState::default()
        .with_vertex_buffer_descriptions(buffers)
        .with_vertex_attributes(attributes)
}

pub trait PipelineManager {
    fn init(&mut self, window: &UIWindow) -> Result<(), Error>;
    fn copy(
        &mut self,
        window: &UIWindow,
        command: &CommandBuffer,
        pass: &CopyPass,
    ) -> Result<(), Error>;
    fn render(
        &self,
        window: &UIWindow,
        command: &CommandBuffer,
        pass: &RenderPass,
    ) -> Result<(), Error>;
    fn finish(&mut self) -> Result<(), Error>;
}

pub struct Pipeline<V, I> {
    pub pipeline: GraphicsPipeline,
    vertex_buffer: BufferHelper<V>,
    instance_buffer: BufferHelper<I>,
}

/// Basic wrapper for a graphics pipeline.
///
/// This class is highly specialized, although I'm not sure if this is
/// necessary, it would be nice to convert this class to work in more generic
/// use cases.
impl<V: Copy, I: Copy> Pipeline<V, I> {
    pub fn new(
        window: &Window,
        device: &Device,
        vertex_shader: &Shader,
        vertex_input: VertexInputState,
        fragment_shader: &Shader,
        vertex_buffer: BufferHelper<V>,
        instance_buffer: BufferHelper<I>,
    ) -> Result<Self, Error> {
        let pipeline = device
            .create_graphics_pipeline()
            .with_target_info(
                GraphicsPipelineTargetInfo::default().with_color_target_descriptions(&[
                    ColorTargetDescription::default()
                        .with_blend_state(
                            ColorTargetBlendState::default()
                                .with_src_color_blendfactor(BlendFactor::SrcAlpha)
                                .with_dst_color_blendfactor(BlendFactor::OneMinusSrcAlpha)
                                .with_color_blend_op(BlendOp::Add)
                                .with_src_alpha_blendfactor(BlendFactor::One)
                                .with_dst_alpha_blendfactor(BlendFactor::OneMinusSrcAlpha)
                                .with_alpha_blend_op(BlendOp::Add)
                                .with_enable_blend(true),
                        )
                        .with_format(device.get_swapchain_texture_format(&window)),
                ]),
            )
            .with_rasterizer_state(RasterizerState::default().with_cull_mode(CullMode::None))
            .with_primitive_type(PrimitiveType::TriangleList)
            .with_vertex_shader(&vertex_shader)
            .with_vertex_input_state(vertex_input)
            .with_fragment_shader(&fragment_shader)
            .build()?;

        Ok(Self {
            pipeline,
            vertex_buffer,
            instance_buffer,
        })
    }

    pub fn use_vertex_data(&mut self, device: &Device, data: &Vec<V>) {
        self.vertex_buffer.data = data.to_vec();
        self.vertex_buffer.refresh_transfer(device);
    }

    pub fn use_instance_data(&mut self, device: &Device, data: &Vec<I>) {
        self.instance_buffer.data = data.to_vec();
        self.instance_buffer.refresh_transfer(device);
    }

    pub fn copy(&self, copy_pass: &CopyPass) -> Result<(), Error> {
        self.vertex_buffer.upload_transfer(&copy_pass);
        self.instance_buffer.upload_transfer(&copy_pass);

        Ok(())
    }

    pub fn draw(&self, render_pass: &RenderPass) {
        render_pass.bind_graphics_pipeline(&self.pipeline);
        render_pass.bind_vertex_buffers(
            0,
            &[self.vertex_buffer.binding(), self.instance_buffer.binding()],
        );
        render_pass.draw_primitives(
            self.vertex_buffer.data.len(),
            self.instance_buffer.data.len(),
            0,
            0,
        );
    }
}
