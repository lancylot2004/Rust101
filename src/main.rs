use clap::{Parser, ValueEnum};
use minifb::{Key, Window, WindowOptions};
use minifb_fonts::font6x8;
use rust_102::game_of_life::{step_parallel, step_serial, step_workers};
use rust_102::seed::seed;
use std::mem::swap;
use std::process::exit;
use std::thread;
use std::time::Instant;

const ALIVE_COLOUR: u32 = 0xFFFFFF;
const DEAD_COLOUR: u32 = 0x000000;
const TEXT_COLOUR: u32 = 0x00FF00;
const TEXT_HEIGHT: usize = 12;
const FPS_UPDATE_INTERVAL: f64 = 0.5;

#[derive(Copy, Clone, ValueEnum, Debug)]
enum Mode {
    Serial,
    Parallel,
    Workers,
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

    /// Chunk size. Required when using the `workers` mode. Ignored otherwise.
    #[arg(
        short = 'c',
        long,
        required_if_eq("mode", "workers"),
    )]
    chunk_size: Option<usize>,
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
    // window.set_target_fps(240);

    let mut curr_buffer = vec![0u8; width * (height - TEXT_HEIGHT)];
    let mut next_buffer = vec![0u8; width * (height - TEXT_HEIGHT)];
    seed(&mut curr_buffer, width, height - TEXT_HEIGHT);

    let mut pixels = vec![0u32; width * height];

    let text = font6x8::new_renderer(width, height, TEXT_COLOUR);
    let mut frame_count = 0;
    let mut last_time = Instant::now();
    let mut fps = 0.0;

    let num_threads = match cli.mode {
        Mode::Serial => 1,
        Mode::Parallel | Mode::Workers => thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
    };

    while window.is_open() && !window.is_key_down(Key::Escape) {
        match cli.mode {
            Mode::Serial => step_serial(&curr_buffer, &mut next_buffer, width, height - TEXT_HEIGHT),
            Mode::Parallel => step_parallel(&curr_buffer, &mut next_buffer, num_threads, width, height - TEXT_HEIGHT),
            Mode::Workers => step_workers(&curr_buffer, &mut next_buffer, num_threads, cli.chunk_size.unwrap(), width, height - TEXT_HEIGHT),
        }

        swap(&mut curr_buffer, &mut next_buffer);
        for (pixel, &cell) in pixels[width * TEXT_HEIGHT..].iter_mut().zip(curr_buffer.iter()) {
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
        text.draw_text(&mut pixels, 2, 2, &format!("mode: {:?}; fps: {fps:.2}; num_threads: {num_threads}", cli.mode));
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
