use crate::{bounds::Bounds, vec2::Vec2};
use std::collections::HashMap;

#[repr(align(256))]
pub struct SimdBlock {
    // Size: 8*8*4 = 256
    data: [[f32; 8]; 8],
}

pub struct Channels {
    channels: HashMap<u32, Channel>,
}

pub struct Channel {
    /// The dimensions of the image data
    size: Vec2<u16>,
    /// The image data
    data: Box<[SimdBlock]>,
}

pub struct ChannelBus {
    translate: Vec2<i16>,
    bounds: Bounds<i16>,
    channels: Vec<u32>,
}
