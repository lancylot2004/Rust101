use std::sync::Mutex;

use crate::image_array::Pixel;

pub struct SingleThreadData<'a> {
    pub rows: &'a mut [Vec<Pixel>],
}

pub struct ByRowThreadData<'a> {
    pub y: usize,
    pub row: &'a mut [Pixel],
}

pub struct ByColumnThreadData<'a> {
    pub x: usize,
    pub column: Vec<&'a mut Pixel>,
}

pub struct ByChunkThreadData<'a> {
    pub x: usize,
    pub y: usize,
    pub up_to_down: bool,
    pub chunk_rows: Vec<&'a mut [Pixel]>,
}

pub struct SequentialThreadData<'a> {
    pub mutex_rows: Mutex<(usize, usize, Vec<&'a mut [Pixel]>)>
}
