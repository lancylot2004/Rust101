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

fn neighbor_count(grid: &Vec<bool>, x: usize, y: usize, width: usize, height: usize) -> u8 {
    let x = x as isize;
    let y = y as isize;

    NEIGHBOUR_KERNEL
        .iter()
        .rfold(0, |acc, (dx, dy)| {
            acc + grid[idx(wrap(x + dx, width), wrap(y + dy, height), width)] as u8
        })
}

fn advance_cell(current: bool, neighbor_count: u8) -> bool {
    match (current, neighbor_count) {
        (true, 2) | (true, 3) | (false, 3) => true,
        _ => false,
    }
}

pub fn step_serial(curr_buffer: &Vec<bool>, next_buffer: &mut Vec<bool>, width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let n = neighbor_count(curr_buffer, x, y, width, height);
            next_buffer[idx(x, y, width)] = advance_cell(curr_buffer[idx(x, y, width)], n);
        }
    }
}

pub fn step_parallel(curr_buffer: &Vec<bool>, next_buffer: &mut Vec<bool>, num_threads: &mut usize, width: usize, height: usize, tile_size: usize) {
    *num_threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let tiles_y = (height + tile_size - 1) / tile_size;
    let tiles_per_worker = (tiles_y + *num_threads - 1) / *num_threads;

    thread::scope(|scope| {
        let mut rest: &mut [bool] = next_buffer;

        for worker in 0..*num_threads {
            let ty0 = worker * tiles_per_worker;
            if ty0 >= tiles_y {
                break;
            }
            let ty1 = ((worker + 1) * tiles_per_worker).min(tiles_y);

            let y0 = ty0 * tile_size;
            let y1 = (ty1 * tile_size).min(height);
            let rows = y1 - y0;

            let take = rows * width;
            let (band, tail) = rest.split_at_mut(take);
            rest = tail;

            scope.spawn(move || {
                let tiles_x = (width + tile_size - 1) / tile_size;

                for ty in ty0..ty1 {
                    let y_start = ty * tile_size;
                    let y_end = (y_start + tile_size).min(height);

                    for tx in 0..tiles_x {
                        let x_start = tx * tile_size;
                        let x_end = (x_start + tile_size).min(width);

                        for y in y_start..y_end {
                            let local_y = y - y0;
                            let row = &mut band[local_y * width..local_y * width + width];

                            for x in x_start..x_end {
                                let n = neighbor_count(curr_buffer, x, y, width, height);
                                row[x] = advance_cell(curr_buffer[idx(x, y, width)], n);
                            }
                        }
                    }
                }
            });
        }
    });
}
