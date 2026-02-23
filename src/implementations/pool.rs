use std::sync::{Arc, Barrier, Mutex, RwLock};
use std::{mem, thread};
use crate::{advance_cell, idx, neighbor_count};

pub struct WorkerPool {
    start_barrier: Arc<Barrier>,
    end_barrier: Arc<Barrier>,
    next_job: Arc<Mutex<usize>>,
    threads: Vec<thread::JoinHandle<()>>,
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.end_barrier.wait();
        for thread in self.threads.drain(..) {
            thread.join().expect("Worker thread panicked!");
        }
    }
}

pub fn initialise_pool(
    curr_buffer: Arc<RwLock<Vec<u8>>>,
    next_buffer: Arc<Mutex<Vec<u8>>>,
    num_threads: usize,
    chunk_size: usize,
    width: usize,
    height: usize,
) -> WorkerPool {
    let next_job: Arc<Mutex<usize>> = Arc::new(Mutex::new(0usize));
    let total = width * height;
    let start_barrier = Arc::new(Barrier::new(num_threads + 1));
    let end_barrier = Arc::new(Barrier::new(num_threads + 1));

    let threads = Vec::from_fn(num_threads, |_| {
        let (curr_buffer, next_buffer, next_job) = (Arc::clone(&curr_buffer), Arc::clone(&next_buffer), Arc::clone(&next_job));
        let (start_barrier, end_barrier) = (Arc::clone(&start_barrier), Arc::clone(&end_barrier));
        let mut scratch = vec![0u8; chunk_size];

        thread::spawn(move || loop {
            start_barrier.wait();
            let curr_buffer = curr_buffer.read().unwrap();

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

            end_barrier.wait();
        })
    });

    WorkerPool { start_barrier, end_barrier, next_job, threads }
}

pub fn step_pool(
    pool: &WorkerPool,
    curr_buffer: &Arc<RwLock<Vec<u8>>>,
    next_buffer: &Arc<Mutex<Vec<u8>>>,
) {
    *pool.next_job.lock().unwrap() = 0;

    pool.start_barrier.wait();
    pool.end_barrier.wait();

    let mut curr = curr_buffer.write().unwrap();
    let mut next = next_buffer.lock().unwrap();
    mem::swap(&mut *curr, &mut *next);
}