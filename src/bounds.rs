use std::ops::Add;

use crate::vec2::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds<T> {
    pub offset: Vec2<T>,
    pub size: Vec2<T>,
}

impl<T> Bounds<T>
where
    T: Add<Output = T> + Copy,
{
    pub fn new(offset: Vec2<T>, size: Vec2<T>) -> Self {
        Self { offset, size }
    }

    pub fn left(&self) -> T {
        self.offset.x
    }

    pub fn top(&self) -> T {
        self.offset.y
    }

    pub fn right(&self) -> T {
        self.offset.x + self.size.x
    }

    pub fn bottom(&self) -> T {
        self.offset.y + self.size.y
    }
}
