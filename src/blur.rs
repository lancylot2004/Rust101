use std::thread;
use std::sync::{Arc, Mutex};
use image::RgbImage;
use crate::Implementation;

pub fn process_blur(img: &mut RgbImage, radius: u32, implementation: Implementation) {
    match implementation {
        Implementation::Basic => blur_basic(img, radius),
        Implementation::Concurrent => blur_concurrently(img, radius),
    }
}

pub fn blur_basic(img: &mut RgbImage, radius: u32) {
    let (width, height) = img.dimensions();
    let radius = radius as i32;
    let original = img.clone();

    for y in 0..height {
        for x in 0..width {
            let mut r_sum: u32 = 0;
            let mut g_sum: u32 = 0;
            let mut b_sum: u32 = 0;
            let mut count: u32 = 0;

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = (x as i32 + dx).max(0).min(width as i32 - 1) as u32;
                    let ny = (y as i32 + dy).max(0).min(height as i32 - 1) as u32;

                    let pixel = original.get_pixel(nx, ny);
                    r_sum += pixel[0] as u32;
                    g_sum += pixel[1] as u32;
                    b_sum += pixel[2] as u32;
                    count += 1;
                }
            }

            img.put_pixel(
                x,
                y,
                image::Rgb([
                    (r_sum / count) as u8,
                    (g_sum / count) as u8,
                    (b_sum / count) as u8,
                ]),
            );
        }
    }
}

pub fn blur_concurrently(img: &mut RgbImage, radius: u32) {
    let (width, height) = img.dimensions();
    let radius = radius as i32;

    let threads = thread::available_parallelism().map(|i| i.get()).unwrap_or(1);

    let original = Arc::new(img.clone());
    let output = Arc::new(Mutex::new(RgbImage::new(width, height)));

    let rows_per_thread = (height as usize + threads - 1) / threads;

    thread::scope(|scope| {
        for t in 0..threads {
            let original = original.clone();
            let output = output.clone();

            scope.spawn(move || {
                let start_y = (t * rows_per_thread) as u32;
                let end_y = ((t + 1) * rows_per_thread) as u32;
                let end_y = end_y.min(height);

                let mut row_pixels: Vec<image::Rgb<u8>> = Vec::with_capacity(width as usize);

                for y in start_y..end_y {
                    row_pixels.clear();

                    for x in 0..width {
                        let mut r_sum: u32 = 0;
                        let mut g_sum: u32 = 0;
                        let mut b_sum: u32 = 0;
                        let mut count: u32 = 0;

                        for dy in -radius..=radius {
                            for dx in -radius..=radius {
                                let nx = (x as i32 + dx).max(0).min(width as i32 - 1) as u32;
                                let ny = (y as i32 + dy).max(0).min(height as i32 - 1) as u32;

                                let pixel = original.get_pixel(nx, ny);
                                r_sum += pixel[0] as u32;
                                g_sum += pixel[1] as u32;
                                b_sum += pixel[2] as u32;
                                count += 1;
                            }
                        }

                        row_pixels.push(image::Rgb([
                            (r_sum / count) as u8,
                            (g_sum / count) as u8,
                            (b_sum / count) as u8,
                        ]));
                    }

                    let mut output_guard = output.lock().unwrap();
                    for (x, pixel) in row_pixels.iter().enumerate() {
                        output_guard.put_pixel(x as u32, y, *pixel);
                    }
                }
            });
        }
    });

    let final_output = Arc::try_unwrap(output)
        .unwrap()
        .into_inner()
        .unwrap();

    *img = final_output;
}

