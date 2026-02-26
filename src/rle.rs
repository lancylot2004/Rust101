/// Run-length-encoding (RLE) to load custom seeds.
///
/// This entire file was essentially written by ChatGPT. Teehee.

use std::fs;
use std::io;
use std::path::PathBuf;


/// Writes an RLE pattern into `grid` (row-major 1D), centered within (width,height).
/// Assumes `grid.len() == width * height`.
pub fn decode_rle_into_centered(
    path: PathBuf,
    grid: &mut Vec<u8>,
    width: usize,
    height: usize,
) -> io::Result<()> {
    let text = fs::read_to_string(path)?;
    grid.fill(0);

    // ---- extract header + payload ----
    let mut header: Option<String> = None;
    let mut payload = String::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if header.is_none() && line.contains('x') && line.contains('y') && line.contains('=') {
            header = Some(line.to_owned());
            continue;
        }
        payload.push_str(line);
    }

    let header = header.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing RLE header"))?;
    let (pat_w, pat_h) = parse_rle_header_xy(&header)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid RLE header"))?;
    assert!(pat_w <= width && pat_h <= height, "pattern too big for target grid");

    // Center the pattern's bounding box within the target grid.
    let ox: isize = (width as isize) / 2 - (pat_w as isize) / 2;
    let oy: isize = (height as isize) / 2 - (pat_h as isize) / 2;

    // ---- decode payload ----
    let mut x: isize = 0;
    let mut y: isize = 0;
    let mut run: usize = 0;

    for ch in payload.chars() {
        match ch {
            '0'..='9' => run = run * 10 + (ch as u8 - b'0') as usize,

            'b' => {
                x += run.max(1) as isize;
                run = 0;
            }

            'o' => {
                let n = run.max(1);
                for _ in 0..n {
                    let gx = ox + x;
                    let gy = oy + y;
                    if (0..width as isize).contains(&gx) && (0..height as isize).contains(&gy) {
                        grid[gx as usize + width * (gy as usize)] = 1;
                    }
                    x += 1;
                }
                run = 0;
            }

            '$' => {
                y += run.max(1) as isize;
                x = 0;
                run = 0;
            }

            '!' => break,
            _ => {}
        }
    }

    Ok(())
}

fn parse_rle_header_xy(header: &str) -> Option<(usize, usize)> {
    // Typical: "x = 398, y = 405, rule = B3/S23"
    fn parse_key_usize(s: &str, key: &str) -> Option<usize> {
        let i = s.find(key)?;
        let rest = &s[i + key.len()..];
        let rest = rest.trim_start();
        let rest = rest.strip_prefix('=')?.trim_start();
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse().ok()
    }
    Some((parse_key_usize(header, "x")?, parse_key_usize(header, "y")?))
}
