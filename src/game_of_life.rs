use std::sync::{Arc, Mutex};
use std::thread;

const NEIGHBOUR_KERNEL: [(isize, isize); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub fn wrap(value: isize, max: usize) -> usize {
    value.rem_euclid(max as isize) as usize
}

fn idx(x: usize, y: usize, width: usize) -> usize {
    y * width + x
}

fn neighbor_count(grid: &[u8], x: usize, y: usize, width: usize, height: usize) -> u8 {
    let x = x as isize;
    let y = y as isize;

    NEIGHBOUR_KERNEL
        .iter()
        .rfold(0u8, |acc, (dx, dy)| {
            acc + grid[idx(wrap(x + dx, width), wrap(y + dy, height), width)]
        })
}

fn advance_cell(current: u8, neighbor_count: u8) -> u8 {
    match (current, neighbor_count) {
        (1, 2) | (1, 3) | (0, 3) => 1,
        (1, _) | (0, _) => 0,
        _ => unreachable!(),
    }
}

pub fn step_serial(curr_buffer: &[u8], next_buffer: &mut [u8], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let n = neighbor_count(curr_buffer, x, y, width, height);
            next_buffer[idx(x, y, width)] = advance_cell(curr_buffer[idx(x, y, width)], n);
        }
    }
}

pub fn step_parallel(
    curr_buffer: &[u8],
    next_buffer: &mut [u8],
    num_threads: usize,
    width: usize,
    height: usize,
) {
    let total = width * height;
    let cells_per_worker = (total + num_threads - 1) / num_threads;

    thread::scope(|scope| {
        let mut left: &mut [u8] = next_buffer;

        for worker_id in 0..num_threads {
            let start = worker_id * cells_per_worker;
            if start >= total {
                break;
            }
            let end = (start + cells_per_worker).min(total);

            let len = end - start;
            let (band, tail) = left.split_at_mut(len);
            left = tail;

            scope.spawn(move || {
                for (i, cell) in (start..end).enumerate() {
                    let (x, y) = (cell % width, cell / width);
                    let n = neighbor_count(curr_buffer, x, y, width, height);
                    band[i] = advance_cell(curr_buffer[idx(x, y, width)], n);
                }
            });
        }
    });
}

pub fn step_workers(
    curr_buffer: &[u8],
    next_buffer: &mut [u8],
    num_threads: usize,
    chunk_size: usize,
    width: usize,
    height: usize,
) {
    let total = width * height;

    let curr_buffer: Arc<[u8]> = Arc::from(curr_buffer);
    let next_buffer: Arc<Mutex<&mut [u8]>> = Arc::new(Mutex::new(next_buffer));
    let next_job: Arc<Mutex<usize>> = Arc::new(Mutex::new(0usize));

    thread::scope(|scope| {
        for _ in 0..num_threads {
            let (curr_buffer, next_buffer, next_job) = (Arc::clone(&curr_buffer), Arc::clone(&next_buffer), Arc::clone(&next_job));
            let mut scratch = vec![0u8; chunk_size];

            scope.spawn(move || {
                loop {
                    let mut next_job = next_job.lock().unwrap();
                    if *next_job >= total {
                        break;
                    }
                    let (start, end) = (*next_job, (*next_job + chunk_size).min(total));
                    *next_job = end;
                    drop(next_job);

                    for (index, cell) in (start..end).enumerate() {
                        let x = cell % width;
                        let y = cell / width;
                        let n = neighbor_count(&curr_buffer, x, y, width, height);
                        scratch[index] = advance_cell(curr_buffer[idx(x, y, width)], n);
                    }

                    next_buffer.lock().unwrap()[start..end].copy_from_slice(&scratch[..(end - start)]);
                }
            });
        }
    });
}
