use crate::{
    geom::{Color, Ray, Point},
    scene::{Camera, DrawableObject, Light}, debug::DebugContext,
};

pub struct Viewport {
    pub height: usize,
    pub width: usize,
    pub values: Vec<Color>,
}

struct ViewportPosition {
    row: usize,
    col: usize,
    idx: usize
}

impl Viewport {
    pub fn new(height: usize, width: usize) -> Viewport {
        Viewport {
            height,
            width,
            values: vec![
                Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0
                };
                height * width
            ],
        }
    }

    fn compute_each<F>(&mut self, compute_fn: F)
    where
        F: Fn(ViewportPosition) -> Color,
    {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = row * self.width + col;

                self.values[idx] = compute_fn(ViewportPosition { row, col, idx });
            }
        }
    }

    pub fn backward_trace(
        &mut self,
        camera: &Camera,
        light: &Light,
        objects: &Vec<DrawableObject>,
        _debug_context: &Option<DebugContext>,
        min_brightness: f64
    ) {
        let total_rows = self.height;
        let total_cols = self.width;

        self.compute_each(|ViewportPosition { row, col, .. }| {
            let prim_ray = compute_prim_ray(camera, row, total_rows, col, total_cols);

            // compute a possible hit
            let mut possible_hit = Option::<(Point, &DrawableObject)>::None;
            let mut min_dist = f64::MAX;
            for obj in objects {
                if let Some(hit_position) = obj.find_intersection(&prim_ray, 0.1) {
                    let dist = hit_position.dist(&camera.eye_position);
                    if dist < min_dist {
                        possible_hit = Some((hit_position, obj));
                        min_dist = dist;
                    }
                }
            }

            possible_hit
                .and_then(|(hit_position, hit_obj)| {
                    let (shadow_ray, _) = Ray::between(&hit_position, &light.position);
                    let is_in_shadow = objects
                        .iter()
                        .map(|obj| obj.find_intersection(&shadow_ray, 0.1))
                        .any(|v| v.is_some());

                    if !is_in_shadow {
                        let brightness = compute_light_brightness(&shadow_ray, hit_obj, light);
                        Some(hit_obj.color.with_brightness(f64::max(brightness, min_brightness)))
                    } else {
                        Some(hit_obj.color.with_brightness(min_brightness))
                    }
                })
                .unwrap_or_else(Color::black)
        })
    }
}

fn compute_light_brightness(shadow_ray: &Ray, drawable: &DrawableObject, light: &Light) -> f64 {
    let perpendicular = drawable.find_perpendicular(&shadow_ray.start);
    let dot = shadow_ray.direction.dot(&perpendicular.direction);
    light.brightness * (dot.powi(3) + dot)
}

fn compute_prim_ray(
    camera: &Camera,
    row: usize,
    total_rows: usize,
    col: usize,
    total_cols: usize,
) -> Ray {
    let vertical_unit = &(&camera.bottom_left - &camera.top_left) / (total_rows as f64);
    let horizontal_unit = &(&camera.top_right - &camera.top_left) / (total_cols as f64);

    let pixel_position = &camera.top_left
        + &(&(&horizontal_unit * (col as f64 + 0.5)) + &(&vertical_unit * (row as f64 + 0.5)));

    Ray::between(&camera.eye_position, &pixel_position).0
}
