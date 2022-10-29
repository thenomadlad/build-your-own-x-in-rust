#![allow(dead_code)]

use append_only_vec::AppendOnlyVec;

use crate::geom::{Color, Point};

pub struct DebugContext {
    pub point_pairs: AppendOnlyVec<(Point, Color)>
}

impl DebugContext {
    pub fn add_reference(&self, viewport_point: &Point, color: Color) {
        self.point_pairs.push((viewport_point.clone(), color));
    }

    pub(crate) fn new() -> Self {
        DebugContext { point_pairs: AppendOnlyVec::new() }
    }
}
