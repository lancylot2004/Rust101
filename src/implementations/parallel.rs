use std::thread;
use crate::{advance_cell, idx, neighbor_count};

/// A parallel step divides the grid into [num_threads] contiguous bands of cells, and each thread
/// processes one band. Using [core::slice::split_at_mut] allows each thread to write to its own 
/// portion of the [next_buffer] without needing synchronization primitives.
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
