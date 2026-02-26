//! Various seed patterns for the Game of Life.

use crate::wrap;

pub fn seed(grid: &mut [u8], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            grid[x + width * y] = 0;
        }
    }

    let cx = (width / 2) as isize;
    let cy = (height / 2) as isize;

    let glider = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
    for (dx, dy) in glider {
        let x = wrap(cx + dx, width);
        let y = wrap(cy + dy, height);
        grid[y + width * x] = 1;
    }

    let r_pentomino = [(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)];
    for (dx, dy) in r_pentomino {
        let x = wrap(cx - 60 + dx, width);
        let y = wrap(cy - 20 + dy, height);
        grid[y + width * x] = 1;
    }
}

/// Gosper glider gun, a "replicator-type" large pattern that produces repeated gliders; anchored
/// 1/4 from left edge, vertically centered.
pub fn seed_gosper(grid: &mut [u8], width: usize, height: usize) {
    grid.fill(0);

    let ox = (width / 4) as isize;
    let oy = (height / 2) as isize;

    const GUN: &[(isize, isize)] = &[
        // left square
        (0, 4), (0, 5), (1, 4), (1, 5),
        // left body
        (10, 4), (10, 5), (10, 6),
        (11, 3), (11, 7),
        (12, 2), (12, 8),
        (13, 2), (13, 8),
        (14, 5),
        (15, 3), (15, 7),
        (16, 4), (16, 5), (16, 6),
        (17, 5),
        // right body
        (20, 2), (20, 3), (20, 4),
        (21, 2), (21, 3), (21, 4),
        (22, 1), (22, 5),
        (24, 0), (24, 1), (24, 5), (24, 6),
        // far right
        (34, 2), (34, 3), (35, 2), (35, 3),
    ];

    for &(dx, dy) in GUN {
        let x = wrap(ox + dx, width);
        let y = wrap(oy + dy - 4, height); // center vertically by shifting pattern height
        grid[x + width * y] = 1;
    }
}
