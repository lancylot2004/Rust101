use clap::Parser;
use minifb::{Key, Window, WindowOptions};
use minifb_fonts::font6x8;
use rust_102::game_of_life::{step_parallel, step_serial};
use rust_102::seed::seed;
use std::mem::swap;
use std::process::exit;
use std::time::Instant;

const ALIVE_COLOUR: u32 = 0xFFFFFF;
const DEAD_COLOUR: u32 = 0x000000;
const TEXT_COLOUR: u32 = 0x00FF00;
const TEXT_HEIGHT: usize = 12;
const FPS_UPDATE_INTERVAL: f64 = 0.5;

#[derive(Parser)]
#[command(
    name = "automata",
    version = env!("CARGO_PKG_VERSION"),
)]
struct CLI {
    /// Window size in pixels.
    #[arg(short, long, value_parser = parse_window_size, default_value = "800x600")]
    size: (usize, usize),

    /// Use parallel processing.
    #[arg(short, long)]
    parallel: bool,

    /// Tile size for parallel processing.
    #[arg(short, long, default_value_t = 16)]
    tile_size: usize,

    #[arg(short, long)]
    /// Debug printing.
    debug: bool,
}

fn main() {
    let cli = CLI::parse();
    let (width, height) = cli.size;

    let mut window = Window::new(
        "Game of Life",
        width,
        height,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
        .expect("Window could not be created.");
    window.set_target_fps(240);

    let mut curr_buffer = vec![false; width * (height - TEXT_HEIGHT)];
    let mut next_buffer = vec![false; width * (height - TEXT_HEIGHT)];
    seed(&mut curr_buffer, width, height - TEXT_HEIGHT);

    let mut pixels = vec![0u32; width * height];

    let text = font6x8::new_renderer(width, height, TEXT_COLOUR);
    let mut frame_count = 0;
    let mut last_time = Instant::now();
    let mut fps = 0.0;
    let mut num_threads = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if cli.parallel {
            step_parallel(&curr_buffer, &mut next_buffer, &mut num_threads, width, height - TEXT_HEIGHT, cli.tile_size);
        } else {
            step_serial(&curr_buffer, &mut next_buffer, width, height - TEXT_HEIGHT);
        }

        swap(&mut curr_buffer, &mut next_buffer);
        for (pixel, &cell) in pixels[width * TEXT_HEIGHT..].iter_mut().zip(curr_buffer.iter()) {
            *pixel = if cell { ALIVE_COLOUR } else { DEAD_COLOUR };
        }

        frame_count += 1;
        let elapsed = last_time.elapsed().as_secs_f64();
        if elapsed >= FPS_UPDATE_INTERVAL {
            fps = frame_count as f64 / elapsed;
            frame_count = 0;
            last_time = Instant::now();
        }

        pixels[..width * TEXT_HEIGHT].fill(0);
        text.draw_text(&mut pixels, 2, 2, &format!("parallel: {}; fps: {fps:.2}; num_threads: {num_threads}", cli.parallel));
        window.update_with_buffer(&pixels, width, height).unwrap();
    }

    exit(0);
}

fn parse_window_size(s: &str) -> Result<(usize, usize), String> {
    let mut parts = s.split('x');
    let width = parts
        .next()
        .ok_or_else(|| "Missing width.".to_string())?
        .parse::<usize>()
        .map_err(|e| format!("Invalid width: {e}"))?;
    let height = parts
        .next()
        .ok_or_else(|| "Missing height.".to_string())?
        .parse::<usize>()
        .map_err(|e| format!("Invalid height: {e}"))?;
    Ok((width, height))
}
