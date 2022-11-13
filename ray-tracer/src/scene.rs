use crate::geom::{Color, Point, Ray};

#[derive(Debug)]
pub enum Shape {
    Sphere { radius: f64 },
}

pub struct Light {
    pub position: Point,
    pub brightness: f64,
}

pub struct Camera {
    pub eye_position: Point,
    pub top_left: Point,
    pub top_right: Point,
    pub bottom_left: Point,
    pub bottom_right: Point,
}

#[derive(Debug)]
pub struct DrawableObject {
    pub id: String,
    pub color: Color,
    pub center: Point,
    pub shape: Shape,
}

impl DrawableObject {
    /// A [Ray] can be represented as:
    ///
    ///     start + direction * t
    ///
    /// depending on the shape, the value can be computed differently
    ///
    /// ## [Shape::Sphere]
    ///
    /// If the ray intersects a sphere with a center C and radius r, then for some t the ray must be distance r from
    /// the center.
    ///
    ///     | (start + direction * t) - C | = r   for some t
    ///
    /// Which we will shorten as:
    ///     | (S + D * t) - C | = r   for some t
    ///
    ///     | (S + D * t) - C | = r   for some t
    ///     | ((S - C) + D * t |^2 = r^2
    ///     ((S - C) + D * t) . ((S - C) + D * t) = r^2
    ///     t^2 (D . D) + 2t (D . (S - C)) + ((S - C) . (S - C)) = r^2
    ///
    /// Thus, we can find the t points that intersected the sphere because it satisfies a quadratic equation:
    ///
    ///     t^2 (D . D) + 2t (D . (S - C)) + (((S - C) . (S - C)) - r^2) = 0
    ///
    pub fn find_intersection(&self, ray: &Ray, min_dist: f64) -> Option<Point> {
        match self.shape {
            #[allow(non_snake_case)]
            Shape::Sphere { radius } => {
                let S_minus_C = &ray.start - &self.center;

                // params for solving
                let D_dot_D = ray.direction.dot(&ray.direction);
                let D_dot_SmC = ray.direction.dot(&S_minus_C);
                let SmC_dot_SmC = S_minus_C.dot(&S_minus_C);

                // solve for t and get the smallest value > 0
                let t = match roots::find_roots_quadratic(
                    D_dot_D,
                    2.0 * D_dot_SmC,
                    SmC_dot_SmC - radius.powi(2),
                ) {
                    roots::Roots::No(_) => None,
                    roots::Roots::One(ts) => if ts[0] > min_dist {
                        Some(ts[0])
                    } else {
                        None
                    },
                    roots::Roots::Two(ts) => ts.into_iter().filter(|t| *t > min_dist).nth(0),
                    _ => unreachable!(),
                };

                // we are only interested in the intersection points after the start
                t.and_then(|val| if val > 0.0 { Some(val) } else { None })
                    .map(|t| &ray.start + &(&ray.direction * t))
            }
        }
    }

    pub fn find_perpendicular(&self, hit_point: &Point) -> Ray {
        match self.shape {
            Shape::Sphere { .. } => Ray::between(&self.center, hit_point).0,
        }
    }
}
