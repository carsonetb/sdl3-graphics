use sdl3::{
    Error,
    gpu::{
        Device, GraphicsPipeline, Shader, VertexAttribute, VertexBufferDescription,
        VertexElementFormat, VertexInputRate, VertexInputState,
    },
    video::Window,
};

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

pub struct Pipeline {
    pipeline: GraphicsPipeline,
}

impl Pipeline {
    pub fn new(
        window: &Window,
        device: &Device,
        vertex_shader: &Shader,
        vertex_input: VertexInputState,
        fragment_shader: &Shader,
    ) -> Result<Self, Error> {
    }
}
