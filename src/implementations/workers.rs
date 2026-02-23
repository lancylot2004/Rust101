use std::sync::{Arc, Mutex};
use std::thread;
use crate::{advance_cell, idx, neighbor_count};

/// Similar to the `parallel` implementation. Each worker thread picks up jobs froma synchronised
/// counter, incremented by [chunk_size]. Each worker uses a scratch buffer, then writes to
/// [next_buffer] when the entire chunk is complete.
pub fn step_workers(
    curr_buffer: &[u8],
    next_buffer: &mut [u8],
    num_threads: usize,
    chunk_size: usize,
    width: usize,
    height: usize,
) {
    let curr_buffer = Arc::new(curr_buffer);
    let next_buffer = Arc::new(Mutex::new(next_buffer));
    let total = width * height;
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
