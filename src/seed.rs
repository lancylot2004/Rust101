//! Various seed patterns for the Game of Life.

use crate::game_of_life::wrap;

pub fn seed(grid: &mut Vec<bool>, width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            grid[x + width * y] = false;
        }
    }

    let cx = (width / 2) as isize;
    let cy = (height / 2) as isize;

    let glider = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
    for (dx, dy) in glider {
        let x = wrap(cx + dx, width);
        let y = wrap(cy + dy, height);
        grid[y + width * x] = true;
    }

    let r_pentomino = [(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)];
    for (dx, dy) in r_pentomino {
        let x = wrap(cx - 60 + dx, width);
        let y = wrap(cy - 20 + dy, height);
        grid[y + width * x] = true;
    }
}
