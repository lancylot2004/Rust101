use std::iter::repeat_with;

use image::GenericImageView;
use image::ImageError;
use image::ImageReader;

pub enum ImageLoadError {
    IoErr(std::io::Error),
    ImageLibErr(ImageError),
}

#[derive(Debug, Default, Clone)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl From<image::Rgba<u8>> for Pixel {
    fn from(value: image::Rgba<u8>) -> Self {
        let [red, green, blue, alpha] = value.0;
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ImageArray(pub Vec<Vec<Pixel>>);

impl ImageArray {
    fn add_pixel(&mut self, x: u32, y: u32, pixel: Pixel) {
        let rows = &mut self.0;
        // Extend the image with empty rows, so that the image's height is
        // maximised with y.
        let extra_row_count = (y as usize + 1).saturating_sub(rows.len());
        let extra_rows = repeat_with(Vec::new).take(extra_row_count);
        rows.extend(extra_rows);
        let row = &mut rows[y as usize];
        let extra_pixels_count = (x as usize + 1).saturating_sub(row.len());
        let extra_pixels = repeat_with(Pixel::default).take(extra_pixels_count);
        row.extend(extra_pixels);
        row.push(pixel);
    }
}

fn load_image(filename: &str) -> Result<ImageArray, ImageLoadError> {
    let img = ImageReader::open(filename)
        .map_err(ImageLoadError::IoErr)?
        .decode()
        .map_err(ImageLoadError::ImageLibErr)?;
    let mut img_data = ImageArray::default();
    img.pixels()
        .for_each(|(x, y, pixel)| img_data.add_pixel(x, y, pixel.into()));
    Ok(img_data)
}
