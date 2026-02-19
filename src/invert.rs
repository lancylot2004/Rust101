use std::thread;
use image::RgbImage;
use crate::Implementation;

pub fn process_invert(img: &mut RgbImage, implementation: Implementation) {
    match implementation {
        Implementation::Basic => invert_basic(img),
        Implementation::Concurrent => invert_concurrently(img),
    }
}

pub fn invert_basic(img: &mut RgbImage) {
    for b in img.iter_mut() {
        *b = 255u8 - *b;
    }
}

pub fn invert_concurrently(img: &mut RgbImage) {
    let threads = thread::available_parallelism().map(|i| i.get()).unwrap_or(1);

    let buffer: &mut [u8] = img.as_mut();
    let len = buffer.len();
    let mut chunk = (len + threads - 1) / threads;
    chunk -= chunk % 3;

    thread::scope(|scope| {
        let mut rest: &mut [u8] = buffer;
        while !rest.is_empty() {
            let take = chunk.min(rest.len());
            let (this, that) = rest.split_at_mut(take);
            rest = that;

            scope.spawn(|| {
                for byte in this {
                    *byte = 255u8 - *byte;
                }
            });
        }
    });
}
