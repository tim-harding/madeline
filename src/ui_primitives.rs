use crate::{color::Color, vec2::Vec2};

pub struct Rect {
    pub position: Vec2<f32>,
    pub size: Vec2<f32>,
    pub corner_radius: f32,
    pub solid_color: Color,
    pub border_color: Color,
}
