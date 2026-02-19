use minifb::{Key, Window, WindowOptions};
use std::thread;
use std::sync::{Arc, Mutex};

const W: usize = 1000;
const H: usize = 700;
const MAX_ITER: u32 = 400;

const TILE: usize = 32;


#[derive(Clone, Copy)]
struct Viewport {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

fn main() {
    let mut window = Window::new("Mandelbrot Set", W, H, WindowOptions::default()).unwrap();
    let vp = Viewport { x_min: -2.2, x_max: 1.0, y_min: -1.2, y_max: 1.2 };

    let mut buffer = vec![0u32; W * H];

    render_parallel_live(&mut window, &mut buffer, vp);
    // render_serial_live(&mut window, &mut buffer, vp);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update();
    }
}

#[inline]
fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}

#[inline]
fn pixel_to_complex(x: usize, y: usize, vp: Viewport) -> (f64, f64) {
    let tx = x as f64 / (W - 1) as f64;
    let ty = y as f64 / (H - 1) as f64;
    (lerp(tx, vp.x_min, vp.x_max), lerp(ty, vp.y_min, vp.y_max))
}

#[inline]
fn mandelbrot(cx: f64, cy: f64, max_iter: u32) -> u32 {
    let (mut x, mut y) = (0.0, 0.0);
    let mut i = 0;

    while i < max_iter {
        let x2 = x * x - y * y + cx;
        let y2 = 2.0 * x * y + cy;
        x = x2;
        y = y2;

        if x * x + y * y > 4.0 {
            break;
        }
        i += 1;
    }
    i
}

#[inline]
fn colour(it: u32) -> u32 {
    if it >= MAX_ITER {
        return 0x000000;
    }
    let t = it as f32 / MAX_ITER as f32;
    let r = (9.0 * (1.0 - t) * t * t * t * 255.0) as u32;
    let g = (15.0 * (1.0 - t) * (1.0 - t) * t * t * 255.0) as u32;
    let b = (8.5 * (1.0 - t) * (1.0 - t) * (1.0 - t) * t * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

fn render_serial_live(window: &mut Window, out: &mut [u32], vp: Viewport) {
    for y in 0..H {
        if !window.is_open() || window.is_key_down(Key::Escape) {
            break;
        }

        for x in 0..W {
            let (cx, cy) = pixel_to_complex(x, y, vp);
            let it = mandelbrot(cx, cy, MAX_ITER);
            out[y * W + x] = colour(it);
        }

        window.update_with_buffer(out, W, H).unwrap();
    }
}

fn render_parallel_live(window: &mut Window, out: &mut [u32], vp: Viewport) {
    let buffer = Arc::new(Mutex::new(out.to_vec()));
    let tile_size = TILE;

    for y_tile_start in (0..H).step_by(tile_size) {
        if !window.is_open() || window.is_key_down(Key::Escape) {
            break;
        }

        let y_tile_end = (y_tile_start + tile_size).min(H);
        let threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        let buffer_clone = buffer.clone();

        thread::scope(|scope| {
            let rows_per_thread = (y_tile_end - y_tile_start + threads - 1) / threads;

            for t in 0..threads {
                let buffer = buffer_clone.clone();
                let y_start = y_tile_start + (t * rows_per_thread);
                if y_start >= y_tile_end {
                    break;
                }
                let y_end = (y_start + rows_per_thread).min(y_tile_end);

                scope.spawn(move || {
                    for y in y_start..y_end {
                        for x in 0..W {
                            let (cx, cy) = pixel_to_complex(x, y, vp);
                            let it = mandelbrot(cx, cy, MAX_ITER);
                            let mut buf = buffer.lock().unwrap();
                            buf[y * W + x] = colour(it);
                        }
                    }
                });
            }
        });

        let buf = buffer.lock().unwrap();
        window.update_with_buffer(&buf, W, H).unwrap();
    }

    let buf = buffer.lock().unwrap();
    out.copy_from_slice(&buf);
}
