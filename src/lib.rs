#![feature(vec_from_fn)]

pub mod implementations;
pub mod seed;

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

/// Wraps a signed coordinate around the grid dimensions, allowing for toroidal addressing.
fn wrap(value: isize, max: usize) -> usize {
    value.rem_euclid(max as isize) as usize
}

/// Converts 2D coordinates to a 1D index in the grid buffer.
fn idx(x: usize, y: usize, width: usize) -> usize {
    y * width + x
}

/// Counts the number of alive neighbours for the cell at ([x], [y]) in the given [grid].
fn neighbor_count(grid: &[u8], x: usize, y: usize, width: usize, height: usize) -> u8 {
    let x = x as isize;
    let y = y as isize;

    NEIGHBOUR_KERNEL
        .iter()
        .rfold(0u8, |acc, (dx, dy)| {
            let cell = grid[idx(wrap(x + dx, width), wrap(y + dy, height), width)];
            debug_assert!(cell == 0 || cell == 1);
            acc + cell
        })
}

/// Returns the next state of a cell given its current state and the number of alive neighbours.
fn advance_cell(current: u8, neighbor_count: u8) -> u8 {
    match (current, neighbor_count) {
        (1, 2) | (1, 3) | (0, 3) => 1,
        (1, _) | (0, _) => 0,
        _ => unreachable!(),
    }
}
