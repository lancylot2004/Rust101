use crate::{advance_cell, idx, neighbor_count};

/// A serial step is cell-wise, left-to-right, top-to-bottom.
pub fn step_serial(curr_buffer: &[u8], next_buffer: &mut [u8], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let n = neighbor_count(curr_buffer, x, y, width, height);
            next_buffer[idx(x, y, width)] = advance_cell(curr_buffer[idx(x, y, width)], n);
        }
    }
}
