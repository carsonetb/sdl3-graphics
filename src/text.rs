use std::collections::HashMap;

use etagere::{AtlasAllocator, Size};
use fontsdf::Font;
use glam::{Mat4, Vec2};
use sdl3::{
    Error,
    gpu::{
        CommandBuffer, CompareOp, CopyPass, Device, Filter, RenderPass, SampleCount, Sampler,
        SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode, ShaderFormat, ShaderStage,
        Texture, TextureCreateInfo, TextureFormat, TextureRegion, TextureSamplerBinding,
        TextureTransferInfo, TextureType, TextureUsage, TransferBufferUsage, VertexElementFormat,
    },
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
#[derive(Debug, Clone, Copy)]
pub struct TextVertexData {
    pos: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TextInstanceData {
    pub pos_scale: [f32; 4],
    pub uv_rect: [f32; 4],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlyphData {
    pub uv_rect: [f32; 4],
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub advance: f32,
}

pub struct FontAtlas {
    pub atlas: Vec<u8>,
    pub texture_width: u32,
    pub texture_height: u32,

    pub glyphs: HashMap<char, GlyphData>,
}

impl FontAtlas {
    pub fn build(font_bytes: &[u8], characters: &str, font_size: f32, atlas_size: u32) -> Self {
        let font = Font::from_bytes(font_bytes).expect("Failed to load font.");

        let mut atlas = vec![0u8; (atlas_size * atlas_size) as usize];
        let mut allocator = AtlasAllocator::new(Size::new(atlas_size as i32, atlas_size as i32));
        let mut glyphs = HashMap::new();

        for char in characters.chars() {
            let (metrics, sdf) = font.rasterize_sdf(char, font_size);

            if metrics.width == 0 || metrics.height == 0 {
                glyphs.insert(
                    char,
                    GlyphData {
                        uv_rect: [0.0, 0.0, 0.0, 0.0],
                        width: 0.0,
                        height: 0.0,
                        offset_x: 0.0,
                        offset_y: 0.0,
                        advance: metrics.advance_width as f32,
                    },
                );
                continue;
            }

            let padding = 2;
            let slot = allocator
                .allocate(Size::new(
                    metrics.width as i32 + padding * 2,
                    metrics.height as i32 + padding * 2,
                ))
                .expect("Font texture atlas is too small.");

            let rect = slot.rectangle;

            let dest_x = (rect.min.x + padding) as u32;
            let dest_y = (rect.min.y + padding) as u32;

            for row in 0..metrics.height {
                // all this is per row.
                let glyph_start = row * metrics.width;
                let glyph_end = glyph_start + metrics.width;
                let glyph_slice = &sdf[glyph_start..glyph_end];

                let atlas_start = ((dest_y + row as u32) * atlas_size + dest_x) as usize;
                let atlas_end = atlas_start + metrics.width;
                let atlas_slice = &mut atlas[atlas_start..atlas_end];

                atlas_slice.copy_from_slice(glyph_slice);
            }

            let tlu = dest_x as f32 / atlas_size as f32;
            let tlv = dest_y as f32 / atlas_size as f32;
            let bru = (dest_x + metrics.width as u32) as f32 / atlas_size as f32;
            let brv = (dest_y + metrics.height as u32) as f32 / atlas_size as f32;

            glyphs.insert(
                char,
                GlyphData {
                    uv_rect: [tlu, tlv, bru, brv],
                    width: metrics.width as f32,
                    height: metrics.height as f32,
                    offset_x: metrics.xmin as f32,
                    offset_y: metrics.ymin as f32,
                    advance: metrics.advance_width,
                },
            );
        }

        Self {
            atlas,
            texture_width: atlas_size,
            texture_height: atlas_size,
            glyphs,
        }
    }
}

pub struct TextPipelineManager {
    initialized: bool,
    texture: Texture<'static>,
    atlas: FontAtlas,
    data: Vec<TextInstanceData>,
    pipeline: Option<Pipeline<TextVertexData, TextInstanceData>>,
    sampler: Sampler,
}

impl TextPipelineManager {
    pub fn new(device: &Device) -> Result<Self, Error> {
        let characters: String = (32..127).filter_map(char::from_u32).collect();
        let font_bytes = include_bytes!("../resources/roboto.ttf");
        let atlas = FontAtlas::build(font_bytes, &characters, 32.0, 1024);
        // println!("{}", atlas.atlas.iter().filter(|&&n| n > 0).count());

        let texture = device.create_texture(
            TextureCreateInfo::default()
                .with_type(TextureType::_2D)
                .with_format(TextureFormat::R8Unorm)
                .with_usage(TextureUsage::SAMPLER)
                .with_width(atlas.texture_width)
                .with_height(atlas.texture_height)
                .with_layer_count_or_depth(1)
                .with_num_levels(1)
                .with_sample_count(SampleCount::NoMultiSampling),
        )?;

        let sampler = device.create_sampler(
            SamplerCreateInfo::default()
                .with_min_filter(Filter::Linear)
                .with_mag_filter(Filter::Linear)
                .with_mipmap_mode(SamplerMipmapMode::Linear)
                .with_address_mode_u(SamplerAddressMode::ClampToEdge)
                .with_address_mode_v(SamplerAddressMode::ClampToEdge)
                .with_address_mode_w(SamplerAddressMode::ClampToEdge)
                .with_mip_lod_bias(0.0)
                .with_max_anisotropy(1.0)
                .with_compare_op(CompareOp::Never)
                .with_min_lod(0.0)
                .with_max_lod(0.0)
                .with_enable_anisotropy(false)
                .with_enable_compare(false),
        )?;

        let command_buffer = device.acquire_command_buffer()?;

        let transfer_buffer = device
            .create_transfer_buffer()
            .with_usage(TransferBufferUsage::UPLOAD)
            .with_size(atlas.texture_width * atlas.texture_height)
            .build()?;

        let mut map = transfer_buffer.map(device, false);
        map.mem_mut().copy_from_slice(&atlas.atlas);
        map.unmap();

        let copy = device.begin_copy_pass(&command_buffer)?;

        copy.upload_to_gpu_texture(
            TextureTransferInfo::default()
                .with_transfer_buffer(&transfer_buffer)
                .with_offset(0)
                .with_pixels_per_row(atlas.texture_width)
                .with_rows_per_layer(atlas.texture_height),
            TextureRegion::default()
                .with_texture(&texture)
                .with_width(atlas.texture_width)
                .with_height(atlas.texture_height)
                .with_depth(1),
            false,
        );

        device.end_copy_pass(copy);

        let fence = command_buffer.submit_and_acquire_fence(device)?;
        device.wait_fences(true, &[fence])?;

        Ok(Self {
            initialized: false,
            texture,
            sampler,
            atlas,
            data: vec![],
            pipeline: None,
        })
    }

    pub fn draw(&mut self, text: &str, pos: Vec2, size: f32, color: Color) {
        let mut draw_pos = pos;
        for char in text.chars() {
            let glyph = self.atlas.glyphs.get(&char).unwrap();
            let scale = size / glyph.height;
            let advance = glyph.advance * scale;
            self.data.push(TextInstanceData {
                pos_scale: [
                    draw_pos.x,
                    draw_pos.y,
                    glyph.width * scale,
                    glyph.height * scale,
                ],
                uv_rect: glyph.uv_rect,
                color: color.to_slice(),
            });
            draw_pos.x += advance;
        }
    }
}

impl PipelineManager for TextPipelineManager {
    fn init(&mut self, window: &UIWindow) -> Result<(), Error> {
        if self.initialized {
            panic!("Init called when already initialized!");
        }

        let vertdata = include_bytes!(concat!(env!("OUT_DIR"), "/text.vert.spv"));
        let fragdata = include_bytes!(concat!(env!("OUT_DIR"), "/text.frag.spv"));

        let vertsh = window
            .device
            .create_shader()
            .with_code(ShaderFormat::SPIRV, vertdata, ShaderStage::Vertex)
            .with_entrypoint(c"main")
            .with_uniform_buffers(1)
            .build()?;
        let fragsh = window
            .device
            .create_shader()
            .with_code(ShaderFormat::SPIRV, fragdata, ShaderStage::Fragment)
            .with_entrypoint(c"main")
            .with_samplers(1)
            .build()?;

        let vertices = vec![
            TextVertexData { pos: [0.0, 0.0] },
            TextVertexData { pos: [1.0, 0.0] },
            TextVertexData { pos: [0.0, 1.0] },
            TextVertexData { pos: [1.0, 0.0] },
            TextVertexData { pos: [1.0, 1.0] },
            TextVertexData { pos: [0.0, 1.0] },
        ];
        let instances = vec![];

        let vertices_buffer = BufferHelper::new(&window.device, vertices, 6)?;
        let instances_buffer = BufferHelper::new(&window.device, instances, 100000)?;

        self.pipeline = Some(Pipeline::new(
            &window.window,
            &window.device,
            &vertsh,
            vertex_input_builder(
                &[
                    vertex_buffer::<TextVertexData>(0),
                    instance_buffer::<TextInstanceData>(1),
                ],
                &[
                    vertex_attribute(0, 0, 0, VertexElementFormat::Float2),
                    vertex_attribute(1, 1, 0, VertexElementFormat::Float4),
                    vertex_attribute(1, 2, 16, VertexElementFormat::Float4),
                    vertex_attribute(1, 3, 32, VertexElementFormat::Float4),
                ],
            ),
            &fragsh,
            vertices_buffer,
            instances_buffer,
        )?);

        self.initialized = true;

        Ok(())
    }

    fn copy(
        &mut self,
        window: &UIWindow,
        command: &CommandBuffer,
        pass: &CopyPass,
    ) -> Result<(), Error> {
        if !self.initialized {
            panic!("TextPipelineManager not initialized.");
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
            panic!("TextPipelineManager not initialized.")
        }

        let (width, height) = window.window.size();
        let proj = Mat4::orthographic_rh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);

        pass.bind_fragment_samplers(
            0,
            &[TextureSamplerBinding::default()
                .with_texture(&self.texture)
                .with_sampler(&self.sampler)],
        );
        command.push_vertex_uniform_data(0, &proj);
        self.pipeline.as_ref().unwrap().draw(pass);

        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        if !self.initialized {
            panic!("TextPipelineManager not initialized.")
        }

        self.data.clear();

        Ok(())
    }
}
