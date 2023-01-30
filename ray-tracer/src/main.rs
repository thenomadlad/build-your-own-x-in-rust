use std::time::Duration;

use debug::DebugContext;
use geom::{Color, Point};
use ndarray::{array, Array1};
use ndarray_linalg::Solve;
use scene::{DrawableObject, Shape};
use sdl2::{event::Event, keyboard::Keycode, render::Canvas, video::Window, rect::Rect};

use crate::{
    scene::{Camera, Light},
    tracer::Viewport,
};

mod geom;
mod scene;
mod tracer;
mod debug;

fn main() {
    // scene consists of one sphere
    let scene = vec![
        DrawableObject {
            id: "red sphere".to_owned(),
            color: Color {
                r: 255,
                g: 0,
                b: 0,
                a: 123
            },
            center: Point {
                x: 50.0,
                y: 10.0,
                z: 10.0,
            },
            shape: Shape::Sphere { radius: 10.0 },
        },
        DrawableObject {
            id: "green sphere".to_owned(),
            color: Color {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            },
            center: Point {
                x: 50.0,
                y: -10.0,
                z: -10.0,
            },
            shape: Shape::Sphere { radius: 10.0 },
        },
        DrawableObject {
            id: "blue sphere".to_owned(),
            color: Color {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            },
            center: Point {
                x: 50.0,
                y: 0.0,
                z: 30.0,
            },
            shape: Shape::Sphere { radius: 20.0 },
        },
    ];

    let camera = Camera {
        eye_position: Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        top_left: Point {
            x: 10.0,
            y: 5.0,
            z: 5.0,
        },
        top_right: Point {
            x: 10.0,
            y: -5.0,
            z: 5.0,
        },
        bottom_left: Point {
            x: 10.0,
            y: 5.0,
            z: -5.0,
        },
        bottom_right: Point {
            x: 10.0,
            y: -5.0,
            z: -5.0,
        },
    };

    let mut viewport = Viewport::new(1000, 1000);
    let mut debug_context = Some(DebugContext::new());
    viewport.backward_trace(
        &camera,
        &Light {
            position: Point {
                x: 0.0,
                y: 100.0,
                z: 100.0,
            },
            brightness: 1.0,
        },
        &scene,
        &mut debug_context,
        0.1
    );

    println!("Displaying");
    display_scene_window(&viewport, 1000, 1000, &camera, &debug_context);

    // for chunk in viewport.values.chunks_exact(viewport.width) {
    //     println!("|{}|", chunk.iter().map(|c| (c.r + c.g + c.b) / 3).map(|v| if v > 0 { "O" } else { " " }).collect::<String>())
    // }
}

fn display_scene_window(viewport: &Viewport, height: usize, width: usize, camera: &Camera, debug_context: &Option<DebugContext>) {
    assert!(height >= viewport.height);
    assert!(height >= viewport.width);

    let rect_height = height / viewport.height;
    let rect_width = width / viewport.width;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Example", width as u32, height as u32)
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .present_vsync() //< this means the screen cannot
        // render faster than your display rate (usually 60Hz or 144Hz)
        .build()
        .unwrap();

    // clear canvas
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();

    draw_scene_on_canvas(viewport, rect_height, rect_width, &mut canvas);

    // debug info if needed
    if let Some(debug_context) = debug_context {
        display_debug_points(&debug_context, height, width, &camera, &mut canvas);
    }

    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        // handle any events
        for event in event_pump.poll_iter() {
            match event {
                Event::MouseButtonDown { x, y, .. } => {
                    let index = (y as usize / rect_height) * viewport.width + (x as usize / rect_width);
                    println!("x: {x}, y: {y}: {index}");
                },
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn draw_scene_on_canvas(viewport: &Viewport, rect_height: usize, rect_width: usize, canvas: &mut Canvas<Window>) {
    for row in 0..viewport.height {
        for col in 0..viewport.width {
            let idx = row * viewport.width + col;

            canvas.set_draw_color(&viewport.values[idx]);
            canvas
                .fill_rect(sdl2::rect::Rect::new(
                    (col * rect_width) as i32,
                    (row * rect_height) as i32,
                    rect_width as u32,
                    rect_height as u32,
                ))
                .unwrap();
            // canvas.set_draw_color(sdl2::pixels::Color::YELLOW);
            // canvas.draw_rect(sdl2::rect::Rect::new(
            //     (col * rect_width) as i32,
            //     (row * rect_height) as i32,
            //     rect_width as u32,
            //     rect_height as u32,
            // )).unwrap();
        }
    }
}

/// let r^ be the vector from the camera's eye to the point we want to render
/// let w^ be the vector from top left to top right of the camera viewport
/// let h^ be the vector from top left to bottom right
/// 
/// If we are able to identify how far along w^ and h^ we need to travel to meet r^, we can use that
/// to compute the x and y coordinate we need to draw onto the screen. THat is, solve for x and y where
/// 
///     (top-left-point) + x w^ + y h^ = eye + t r^     x, y, t are real numbers
/// 
/// rearranging the values, we are solving for:
///     
///     x w^ + y h^ - t r^ = eye - (top-left-point)
/// 
/// which is a system of linear equations
fn display_debug_points(debug_context: &DebugContext, height: usize, width: usize, camera: &Camera, canvas: &mut Canvas<Window>) {
    let x_direction = &camera.top_right - &camera.top_left;
    let y_direction = &camera.bottom_left - &camera.top_left;

    let v = &camera.eye_position - &camera.top_left;

    for (point, color) in debug_context.point_pairs.iter() {
        // compute the point to show on the view screen
        let ray = point - &camera.eye_position;
        let solution: Array1<f64> = array![
            [x_direction.x, x_direction.y, x_direction.z],
            [y_direction.x, y_direction.y, y_direction.z],
            [-ray.x, -ray.y, -ray.z],
        ].t().solve_into(array![v.x, v.y, v.z]).unwrap();

        canvas.set_draw_color(color);
        canvas.fill_rect(
            Rect::new(
                (width as f64 * solution.get(0).unwrap()) as i32 - 3,
                (height as f64 * solution.get(1).unwrap()) as i32 - 3,
                6,
                6,
            ),
        ).unwrap();
    }
}
