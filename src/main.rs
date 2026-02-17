use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use image::{DynamicImage, ImageFormat, RgbImage};
use std::fs;
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Parser)]
#[command(
    name = "picture_lib",
    version = env!("CARGO_PKG_VERSION"),
    about = "Applies an image process to an input image and writes an output image.",
)]
struct CLI {
    /// Input image path.
    #[arg(value_parser = valid_input_file)]
    input_path: String,

    /// Output image path.
    #[arg(value_parser = valid_output_file)]
    output_path: String,

    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,

    /// Overwrite output file if it already exists.
    #[arg(long)]
    overwrite: bool,

    #[command(subcommand)]
    process: Process,
}

#[derive(Subcommand)]
enum Process {
    /// Invert RGB channels.
    Invert,

    /// Convert to grayscale.
    Grayscale,

    /// Rotate by 90/180/270 degrees.
    Rotate {
        #[arg(value_enum)]
        degrees: RotateDegrees,
    },

    /// Flip horizontally or vertically.
    Flip {
        #[arg(value_enum)]
        axis: FlipAxis,
    },

    /// Blur the image.
    Blur,
}

#[derive(Clone, Copy, ValueEnum)]
enum RotateDegrees {
    #[value(name = "90")]
    Deg90,
    #[value(name = "180")]
    Deg180,
    #[value(name = "270")]
    Deg270,
}

#[derive(Clone, Copy, ValueEnum)]
enum FlipAxis {
    #[value(name = "h")]
    Horizontal,
    #[value(name = "v")]
    Vertical,
}

fn main() {
    let cli = CLI::parse();

    if !cli.overwrite && fs::metadata(&cli.output_path).is_ok() {
        CLI::command()
            .error(
                ErrorKind::InvalidValue,
                format!(
                    "File {} already exists. Use --overwrite to overwrite it.",
                    cli.output_path
                ),
            )
            .exit();
    }

    let input = image::open(&cli.input_path).unwrap_or_else(|e| {
        CLI::command()
            .error(
                ErrorKind::Io,
                format!("Error reading image {}: {}", cli.input_path, e),
            )
            .exit();
    });

    let rgb = input.to_rgb8();
    let out_rgb = match cli.process {
        Process::Invert => process_invert(rgb, cli.verbose),
        Process::Grayscale => process_grayscale(rgb, cli.verbose),
        Process::Rotate { degrees } => process_rotate(rgb, degrees, cli.verbose),
        Process::Flip { axis } => process_flip(rgb, axis, cli.verbose),
        Process::Blur => process_blur(rgb, cli.verbose),
    }
    .unwrap_or_else(|e| {
        CLI::command()
            .error(
                ErrorKind::InvalidValue,
                format!("Error processing image: {e}"),
            )
            .exit()
    });

    write_image(&cli.output_path, out_rgb).unwrap_or_else(|e| {
        CLI::command()
            .error(
                ErrorKind::Io,
                format!("Error writing image {}: {}", cli.output_path, e),
            )
            .exit();
    });

    exit(0);
}

fn write_image(path: &str, rgb: RgbImage) -> Result<(), String> {
    let fmt = ImageFormat::from_path(path)
        .map_err(|e| format!("Cannot infer output image format from {}: {}", path, e))?;

    DynamicImage::ImageRgb8(rgb)
        .save_with_format(path, fmt)
        .map_err(|e| e.to_string())
}

fn valid_input_file(s: &str) -> Result<String, String> {
    let p = Path::new(s);

    match fs::metadata(p) {
        Err(e) => Err(format!("Error reading file {}: {}", s, e)),
        Ok(md) => md
            .is_file()
            .then_some(s.to_string())
            .ok_or(format!("{} is not a file.", s)),
    }?;

    ImageFormat::from_path(p).map_err(|_| {
        format!("File {s} has an unknown/unsupported extension; cannot infer image format.")
    })?;

    Ok(s.to_string())
}

fn valid_output_file(s: &str) -> Result<String, String> {
    ImageFormat::from_path(s).map_err(|_| {
        format!("Output file {s} has an unknown/unsupported extension; cannot infer image format.")
    })?;

    let path = PathBuf::from(s);
    let mut parent = path
        .parent()
        .ok_or(format!("Cannot determine parent directory of {s}."))?;

    if parent.as_os_str().is_empty() {
        parent = Path::new(".");
    } else {
        parent.exists().not().then_some(()).ok_or(format!(
            "Parent directory {} of {} does not exist.",
            parent.display(),
            s
        ))?;
        parent
            .is_dir()
            .not()
            .then_some(())
            .ok_or(format!("Parent of {s} is not a directory."))?;
    }

    parent
        .metadata()
        .map_err(|e| format!("Error reading directory {}: {}", parent.display(), e))?
        .permissions()
        .readonly()
        .not()
        .then_some(())
        .ok_or(format!(
            "Permission denied, cannot write to {:#?}",
            Path::new(s).file_name().unwrap_or("<unknown>".as_ref())
        ))?;

    Ok(s.to_string())
}

fn process_invert(img: RgbImage, verbose: bool) -> Result<RgbImage, String> {
    if verbose {
        eprintln!("[picture_lib] invert: placeholder (no-op)");
    }
    // TODO: per pixel (r,g,b) -> (255-r, 255-g, 255-b)
    Ok(img)
}

fn process_grayscale(img: RgbImage, verbose: bool) -> Result<RgbImage, String> {
    if verbose {
        eprintln!("[picture_lib] grayscale: placeholder (no-op)");
    }
    // TODO: per pixel -> luminance, set (y,y,y)
    Ok(img)
}

fn process_rotate(
    img: RgbImage,
    degrees: RotateDegrees,
    verbose: bool,
) -> Result<RgbImage, String> {
    if verbose {
        eprintln!("[picture_lib] rotate: placeholder (no-op)");
    }
    let _ = degrees;
    // TODO: rotate by 90/180/270; consider dimension swap for 90/270
    Ok(img)
}

fn process_flip(img: RgbImage, axis: FlipAxis, verbose: bool) -> Result<RgbImage, String> {
    if verbose {
        eprintln!("[picture_lib] flip: placeholder (no-op)");
    }
    let _ = axis;
    // TODO: h = mirror left-right, v = mirror top-bottom
    Ok(img)
}

fn process_blur(img: RgbImage, verbose: bool) -> Result<RgbImage, String> {
    if verbose {
        eprintln!("[picture_lib] blur: placeholder (no-op)");
    }
    // TODO: convolution / box blur / gaussian blur over RGB pixels
    Ok(img)
}
