pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn RGB(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn RGBA(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_slice(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub const WHITE: Self = Self::RGB(1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::RGB(0.0, 0.0, 0.0);
}
