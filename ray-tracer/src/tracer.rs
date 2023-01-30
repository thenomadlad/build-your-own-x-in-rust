use crate::{
    geom::{Color, Ray, Point},
    scene::{Camera, DrawableObject, Light}, debug::DebugContext,
};

pub struct Viewport {
    pub height: usize,
    pub width: usize,
    pub values: Vec<Color>,
}

#[allow(dead_code)]
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
        ambient_brightness: f64
    ) {
        let total_rows = self.height;
        let total_cols = self.width;

        self.compute_each(|ViewportPosition { row, col, .. }| {
            let mut prim_rays = vec![compute_prim_ray(camera, row, total_rows, col, total_cols)];
            let mut color = Color::black();

            while let Some(prim_ray) = prim_rays.pop() {
                if let Some((hit_position, hit_obj)) = find_intersecting_object(objects, &prim_ray) {
                    
                    let (shadow_ray, _) = Ray::between(&hit_position, &light.position);
                    let is_in_shadow = objects
                        .iter()
                        .map(|obj| obj.find_intersection(&shadow_ray, 0.1))
                        .any(|v| v.is_some());

                    if !is_in_shadow {
                        let brightness = compute_light_brightness(&shadow_ray, hit_obj, light);
                        
                        // compute color using brightness and return
                        hit_obj.color.with_brightness(f64::max(brightness, ambient_brightness));
                    } else {
                        // if in shadow, we show with ambient brightness
                        hit_obj.color.with_brightness(ambient_brightness);
                    }
                }
            }

            color
        })
    }
}

fn find_intersecting_object<'a>(objects: &'a Vec<DrawableObject>, ray: &Ray) -> Option<(Point, &'a DrawableObject)> {
    objects.iter()
        .map(|obj| obj.find_intersection(&ray, 0.1).map(|hit_position| (hit_position, obj)))
        .flatten()
        .min_by_key(|(point, _)| (ray.start.dist(point) * 100000.0).round() as u64)
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
