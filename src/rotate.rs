use std::thread;
use image::RgbImage;

pub fn process_rotate(img: &RgbImage, degrees: u16, implementation: crate::Implementation) -> RgbImage {
    match implementation {
        crate::Implementation::Basic => rotate_basic(img, degrees),
        crate::Implementation::Concurrent => rotate_concurrently(img, degrees),
    }
}

#[inline]
fn source_coord_for_out(ox: u32, oy: u32, w: u32, h: u32, deg: u16) -> (u32, u32) {
    match deg {
        90  => (oy, h - 1 - ox),
        180 => (w - 1 - ox, h - 1 - oy),
        270 => (w - 1 - oy, ox),
        _   => (ox, oy),
    }
}

#[inline]
fn rotated_dims(w: u32, h: u32, deg: u16) -> (u32, u32) {
    match deg {
        90 | 270 => (h, w),
        _ => (w, h),
    }
}

pub fn rotate_basic(img: &RgbImage, degrees: u16) -> RgbImage {
    let (w, h) = img.dimensions();
    let (nw, nh) = rotated_dims(w, h, degrees);

    let src = img.as_raw();
    let row_bytes = (nw * 3) as usize;
    let mut out = vec![0u8; (nh as usize) * row_bytes];

    for oy in 0..nh {
        let row = &mut out[(oy as usize) * row_bytes .. (oy as usize + 1) * row_bytes];
        for ox in 0..nw {
            let (sx, sy) = source_coord_for_out(ox, oy, w, h, degrees);
            let s = ((sy * w + sx) * 3) as usize;
            let d = (ox * 3) as usize;
            row[d..d + 3].copy_from_slice(&src[s..s + 3]);
        }
    }

    RgbImage::from_raw(nw, nh, out).unwrap()
}

pub fn rotate_concurrently(img: &RgbImage, degrees: u16) -> RgbImage {
    let (w, h) = img.dimensions();
    let (nw, nh) = rotated_dims(w, h, degrees);

    let src = img.as_raw();
    let row_bytes = (nw * 3) as usize;
    let mut out = vec![0u8; (nh as usize) * row_bytes];

    let threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let threads = threads.min(nh as usize).max(1);

    let rows_per_thread = (nh as usize + threads - 1) / threads;

    thread::scope(|scope| {
        let mut remaining: &mut [u8] = out.as_mut_slice();

        for t in 0..threads {
            let take_rows = rows_per_thread.min((nh as usize).saturating_sub(t * rows_per_thread));
            if take_rows == 0 { break; }

            let take_bytes = take_rows * row_bytes;
            let (mine, rest) = remaining.split_at_mut(take_bytes);
            remaining = rest;

            let start_oy = (t * rows_per_thread) as u32;

            scope.spawn(move || {
                for local_row in 0..take_rows {
                    let oy = start_oy + local_row as u32;
                    let row = &mut mine[local_row * row_bytes .. (local_row + 1) * row_bytes];

                    for ox in 0..nw {
                        let (sx, sy) = source_coord_for_out(ox, oy, w, h, degrees);
                        let s = ((sy * w + sx) * 3) as usize;
                        let d = (ox * 3) as usize;
                        row[d..d + 3].copy_from_slice(&src[s..s + 3]);
                    }
                }
            });
        }
    });

    RgbImage::from_raw(nw, nh, out).unwrap()
}
