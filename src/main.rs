use glam::Mat4;
use sdl3::{
    event::Event,
    gpu::{
        Buffer, BufferBinding, BufferMemMap, BufferRegion, BufferUsageFlags,
        ColorTargetDescription, ColorTargetInfo, CullMode, Device, FillMode,
        GraphicsPipelineTargetInfo, LoadOp, PrimitiveType, RasterizerState, Shader, ShaderFormat,
        ShaderStage, StoreOp, TransferBufferLocation, TransferBufferUsage, VertexAttribute,
        VertexBufferDescription, VertexElementFormat, VertexInputRate, VertexInputState,
    },
    keyboard::Keycode,
    sys::gpu::{SDL_GPULoadOp, SDL_GPUStoreOp},
};
use std::{error::Error, os::raw::c_float};

use sdl3::pixels::Color;

use crate::{
    buffer::BufferHelper,
    pipeline::{instance_buffer, vertex_attribute, vertex_buffer, vertex_input_builder},
    rect::{RectInstanceData, RectVertexData},
};

mod buffer;
mod pipeline;
mod rect;
mod window;

fn main() -> Result<(), Box<dyn Error>> {
    let context = sdl3::init()?;
    let video_subsystem = context.video()?;

    let window = video_subsystem
        .window("Rust SDL3", 800, 600)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let device = Device::new(ShaderFormat::SPIRV, true)?.with_window(&window)?;

    let vs_source = include_bytes!(concat!(env!("OUT_DIR"), "/rect.vert.spv"));
    let fs_source = include_bytes!(concat!(env!("OUT_DIR"), "/rect.frag.spv"));

    let vs_shader = device
        .create_shader()
        .with_code(ShaderFormat::SPIRV, vs_source, ShaderStage::Vertex)
        .with_entrypoint(c"main")
        .with_uniform_buffers(1)
        .build()?;
    let fs_shader = device
        .create_shader()
        .with_code(ShaderFormat::SPIRV, fs_source, ShaderStage::Fragment)
        .with_entrypoint(c"main")
        .build()?;

    let pipeline = device
        .create_graphics_pipeline()
        .with_target_info(
            GraphicsPipelineTargetInfo::new()
                .with_color_target_descriptions(&[ColorTargetDescription::new()
                    .with_format(device.get_swapchain_texture_format(&window))]),
        )
        .with_rasterizer_state(RasterizerState::default().with_cull_mode(CullMode::None))
        .with_fill_mode(FillMode::Fill)
        .with_primitive_type(PrimitiveType::TriangleList)
        .with_vertex_shader(&vs_shader)
        .with_vertex_input_state(vertex_input_builder(
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
        ))
        .with_fragment_shader(&fs_shader)
        .build()?;

    let vertices = vec![
        RectVertexData { pos: [0.0, 0.0] },
        RectVertexData { pos: [1.0, 0.0] },
        RectVertexData { pos: [0.0, 1.0] },
        RectVertexData { pos: [1.0, 0.0] },
        RectVertexData { pos: [1.0, 1.0] },
        RectVertexData { pos: [0.0, 1.0] },
    ];

    let instances = vec![RectInstanceData {
        pos: [10.0, 10.0],
        dim: [500.0, 500.0],
        col: [1.0, 0.0, 0.0, 1.0],
    }];

    let vertices_buffer = BufferHelper::new(&device, vertices, 6)?;
    let instances_buffer = BufferHelper::new(&device, instances, 1000)?;

    let mut event_pump = context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let (width, height) = window.size();
        let proj = Mat4::orthographic_rh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);

        let mut command_buffer = device.acquire_command_buffer()?;

        let copy_pass = device.begin_copy_pass(&command_buffer)?;

        vertices_buffer.upload_transfer(&copy_pass);
        instances_buffer.upload_transfer(&copy_pass);

        device.end_copy_pass(copy_pass);

        if let Ok(swapchain) = command_buffer.wait_and_acquire_swapchain_texture(&window) {
            let color_targets = [ColorTargetInfo::default()
                .with_texture(&swapchain)
                .with_clear_color(Color::RGB(0, 0, 0))
                .with_load_op(SDL_GPULoadOp::CLEAR)
                .with_store_op(SDL_GPUStoreOp::STORE)];

            let render_pass = device.begin_render_pass(&command_buffer, &color_targets, None)?;

            render_pass.bind_graphics_pipeline(&pipeline);
            render_pass
                .bind_vertex_buffers(0, &[vertices_buffer.binding(), instances_buffer.binding()]);
            command_buffer.push_vertex_uniform_data(0, &proj);
            render_pass.draw_primitives(6, 1, 0, 0);

            device.end_render_pass(render_pass);
            command_buffer.submit()?;
        } else {
            command_buffer.cancel();
        }
    }

    Ok(())
}
