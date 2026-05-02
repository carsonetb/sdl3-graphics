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
