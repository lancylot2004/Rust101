use crate::image_array::Pixel;

pub trait PictureEffect {
    fn process_pixel(x: usize, y: usize, pixel: &mut Pixel);
}