use std::ops::{Add, Div, Mul, Sub};

#[derive(Default, Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub fn dot(&self, rhs: &Point) -> f64 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }

    pub fn norm(&self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn dist(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }
}

impl Sub for &Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Add for &Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Div<f64> for &Point {
    type Output = Point;

    fn div(self, rhs: f64) -> Self::Output {
        Point {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Mul<f64> for &Point {
    type Output = Point;

    fn mul(self, rhs: f64) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

#[derive(Debug)]
pub struct Ray {
    pub start: Point,
    pub direction: Point,
}

impl Ray {
    pub fn between(start: &Point, end: &Point) -> (Ray, f64) {
        let diff = end - start;
        let dist = diff.norm();
        let direction = &diff / dist;

        (Ray {
            start: start.clone(),
            direction,
        }, dist)
    }
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn with_brightness(&self, brightness: f64) -> Color {
        Color {
            r: ((self.r as f64 * brightness) % 256.0) as u8,
            g: ((self.g as f64 * brightness) % 256.0) as u8,
            b: ((self.b as f64 * brightness) % 256.0) as u8,
            a: self.a,
        }
    }

    pub fn black() -> Color {
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

impl From<&Color> for sdl2::pixels::Color {
    fn from(color: &Color) -> Self {
        sdl2::pixels::Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}
