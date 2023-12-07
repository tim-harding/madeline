pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_srgb(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: srgb(r),
            g: srgb(g),
            b: srgb(b),
            a: srgb(a),
        }
    }
}

fn srgb(c: u8) -> f32 {
    let c = c as f32 / 255.;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
