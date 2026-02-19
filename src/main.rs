use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use image::{DynamicImage, ImageFormat, RgbImage};
use std::fs;
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::process::exit;
use picture_lib::Implementation;
use picture_lib::invert::process_invert;
use picture_lib::rotate::process_rotate;
use picture_lib::blur::process_blur;

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

    /// Implementation type to use.
    #[arg(default_value = "basic")]
    implementation: Implementation,

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
    Blur {
        radius: u32,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum RotateDegrees {
    #[value(name = "90")]
    Deg90 = 90,
    #[value(name = "180")]
    Deg180 = 180,
    #[value(name = "270")]
    Deg270 = 270,
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

    let mut rgb = input.to_rgb8();
    match cli.process {
        Process::Invert => {
            process_invert(&mut rgb, cli.implementation);
        },
        Process::Grayscale => todo!("Not yet implemented."),
        Process::Rotate { degrees } => {
            let new_image = process_rotate(&mut rgb, degrees as u16, cli.implementation);
            rgb = new_image;
        },
        Process::Flip { .. } => todo!("Not yet implemented."),
        Process::Blur { radius } => {
            process_blur(&mut rgb, radius, cli.implementation);
        },
    };

    write_image(&cli.output_path, rgb).unwrap_or_else(|e| {
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
