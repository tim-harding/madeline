use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub position: Vec3,
    pub size: Vec2,
    pub corner_radius: f32,
    pub border_width: f32,
    pub solid_color: Vec3,
    pub border_color: Vec3,
}
