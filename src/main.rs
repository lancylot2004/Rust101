mod demonstrations;

use clap::{Parser, ValueEnum};
use minifb::{Key, Window, WindowOptions};
use minifb_fonts::font6x8;
use rust_102::implementations::parallel::step_parallel;
use rust_102::implementations::pool::{initialise_pool, step_pool};
use rust_102::implementations::serial::step_serial;
use rust_102::implementations::workers::step_workers;
use rust_102::rle::decode_rle_into_centered;
use rust_102::seed::seed_gosper;
use std::mem::swap;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;

const ALIVE_COLOUR: u32 = 0xFFFFFF;
const DEAD_COLOUR: u32 = 0x000000;
const TEXT_COLOUR: u32 = 0x00FF00;
const TEXT_HEIGHT: usize = 12;
const FPS_UPDATE_INTERVAL: f64 = 0.5;

#[derive(Copy, Clone, ValueEnum, Debug)]
#[derive(PartialEq)]
enum Mode {
    Serial,
    Parallel,
    Workers,
    Pool,
}

#[derive(Parser)]
#[command(
    name = "automata",
    version = env!("CARGO_PKG_VERSION"),
)]
struct CLI {
    /// Window size in pixels.
    #[arg(short, long, value_parser = parse_window_size, default_value = "800x600")]
    size: (usize, usize),

    /// What strategy to use for stepping the simulation.
    #[arg(short, long)]
    mode: Mode,

    /// Chunk size. Required when using the [Workers] or [Pool] mode. Ignored otherwise.
    #[arg(
        short = 'c',
        long,
        required_if_eq_any([("mode", "workers"), ("mode", "pool")]),
    )]
    chunk_size: Option<usize>,

    /// An optional run-length-encoded initial state to replace the default seed.
    #[arg(long, value_name = "FILE")]
    seed: Option<PathBuf>,
}

fn main() {
    let cli = CLI::parse();
    let (width, height) = cli.size;
    let grid_height = height - TEXT_HEIGHT;

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

    let mut pixels = vec![0u32; width * height];

    let drawer = font6x8::new_renderer(width, height, TEXT_COLOUR);
    let mut frame_count = 0;
    let mut last_time = Instant::now();
    let mut fps = 0.0;

    let num_threads = match cli.mode {
        Mode::Serial => 1,
        _ => thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
    };

    let mut render_frame = |window: &mut Window, grid: &[u8]| {
        for (pixel, &cell) in pixels[width * TEXT_HEIGHT..].iter_mut().zip(grid.iter()) {
            *pixel = if cell == 1 { ALIVE_COLOUR } else { DEAD_COLOUR };
        }

        frame_count += 1;
        let elapsed = last_time.elapsed().as_secs_f64();
        if elapsed >= FPS_UPDATE_INTERVAL {
            fps = frame_count as f64 / elapsed;
            frame_count = 0;
            last_time = Instant::now();
        }

        pixels[..width * TEXT_HEIGHT].fill(0);
        let mut text = format!("mode: {:?}; fps: {fps:.2}; num_threads: {num_threads}", cli.mode);
        if cli.mode == Mode::Workers || cli.mode == Mode::Pool {
            let chunk_size = cli.chunk_size.unwrap();
            text.push_str(&format!("; chunk_size: {chunk_size}"));
        }
        drawer.draw_text(&mut pixels, 2, 2, &text);
        window.update_with_buffer(&pixels, width, height).unwrap();
    };

    let init_grid = |buffer: &mut Vec<u8>| {
        match cli.seed {
            Some(path) => decode_rle_into_centered(path, buffer, width, grid_height).expect("Failed to decode RLE file"),
            None => seed_gosper(buffer, width, grid_height),
        }
    };

    match cli.mode {
        Mode::Serial | Mode::Parallel | Mode::Workers => {
            let mut curr_buffer = vec![0u8; width * grid_height];
            let mut next_buffer = vec![0u8; width * grid_height];
            init_grid(&mut curr_buffer);

            while window.is_open() && !window.is_key_down(Key::Escape) {
                match cli.mode {
                    Mode::Serial => step_serial(&curr_buffer, &mut next_buffer, width, grid_height),
                    Mode::Parallel => step_parallel(&curr_buffer, &mut next_buffer, num_threads, width, grid_height),
                    Mode::Workers => step_workers(&curr_buffer, &mut next_buffer, num_threads, cli.chunk_size.unwrap(), width, grid_height),
                    _ => unreachable!("Mode already filtered"),
                }

                swap(&mut curr_buffer, &mut next_buffer);
                render_frame(&mut window, &curr_buffer);
            }
        }
        Mode::Pool => {
            let mut curr_vec = vec![0u8; width * grid_height];
            init_grid(&mut curr_vec);
            let curr_buffer = Arc::new(RwLock::new(curr_vec));
            let next_buffer = Arc::new(Mutex::new(vec![0u8; width * grid_height]));

            let chunk_size = cli.chunk_size.unwrap_or(256);
            let pool = initialise_pool(
                Arc::clone(&curr_buffer),
                Arc::clone(&next_buffer),
                num_threads,
                chunk_size,
                width,
                grid_height,
            );

            while window.is_open() && !window.is_key_down(Key::Escape) {
                step_pool(&pool, &curr_buffer, &next_buffer);
                render_frame(&mut window, &curr_buffer.read().unwrap());
            }
        }
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
