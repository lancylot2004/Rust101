use std::iter::once;
use std::iter::repeat;
use std::iter::repeat_with;

use crate::effect::PictureEffect;
use crate::image_array::ImageArray;
use crate::image_array::Pixel;
use crate::thread_data::ByChunkThreadData;
use crate::thread_data::ByColumnThreadData;
use crate::thread_data::ByRowThreadData;
use crate::thread_data::SingleThreadData;

pub trait ProcessingStrategy {
    type ThreadDatas<'a>;

    fn thread_datas<'a>(
        &self,
        output_array: &'a mut ImageArray,
    ) -> impl Iterator<Item = Self::ThreadDatas<'a>>;

    fn process_data<'a, P: PictureEffect>(thread_data: Self::ThreadDatas<'a>);
}

pub struct SingleThreadedStrategy;

impl ProcessingStrategy for SingleThreadedStrategy {
    type ThreadDatas<'a> = SingleThreadData<'a>;

    fn thread_datas<'a>(
        &self,
        image_array: &'a mut ImageArray,
    ) -> impl Iterator<Item = Self::ThreadDatas<'a>> {
        once(SingleThreadData {
            rows: image_array.0.as_mut_slice(),
        })
    }

    fn process_data<'a, P: PictureEffect>(thread_data: Self::ThreadDatas<'a>) {
        thread_data
            .rows
            .iter_mut()
            .enumerate()
            .map(|(y, row)| (y, row.iter_mut().enumerate()))
            .flat_map(|(y, xs)| repeat(y).zip(xs))
            .for_each(|(y, (x, pixel))| P::process_pixel(x, y, pixel))
    }
}

pub struct ByRowThreadStrategy;

impl ProcessingStrategy for ByRowThreadStrategy {
    type ThreadDatas<'a> = ByRowThreadData<'a>;

    fn thread_datas<'a>(
        &self,
        output_array: &'a mut ImageArray,
    ) -> impl Iterator<Item = Self::ThreadDatas<'a>> {
        output_array
            .0
            .iter_mut()
            .enumerate()
            .map(move |(y, row)| ByRowThreadData { y, row })
    }

    fn process_data<'a, P: PictureEffect>(thread_data: Self::ThreadDatas<'a>) {
        thread_data
            .row
            .iter_mut()
            .enumerate()
            .for_each(|(x, pixel)| P::process_pixel(x, thread_data.y, pixel))
    }
}

/// Mutably borrows the image data and returns each
fn to_vec_of_mutable_rows<'a, T>(rows: &'a mut [Vec<T>]) -> Vec<&'a mut [T]> {
    rows.iter_mut().map(Vec::as_mut_slice).collect()
}

pub struct ByColumnThreadStrategy;

impl ProcessingStrategy for ByColumnThreadStrategy {
    type ThreadDatas<'a> = ByColumnThreadData<'a>;

    fn thread_datas<'a>(
        &self,
        output_array: &'a mut ImageArray,
    ) -> impl Iterator<Item = Self::ThreadDatas<'a>> {
        // First, convert into a list of mutable rows.
        let mut mutable_rows = to_vec_of_mutable_rows(&mut output_array.0);
        // Take the first element from each slice (the column)
        let make_column = move || {
            mutable_rows
                .iter_mut()
                .try_fold(Vec::new(), |mut column, row| {
                    // Try to remove the first element of the row, and add this
                    // to the column to be processed.
                    // If the row is empty, then we assume that all the pixels
                    // in the column are empty and terminate early.
                    let head = row.split_off_first_mut()?;
                    column.push(head);
                    Some(column)
                })
        };

        repeat_with(make_column)
            .flatten()
            .enumerate()
            .map(move |(x, column)| ByColumnThreadData { x, column })
    }

    fn process_data<'a, P: PictureEffect>(mut thread_data: Self::ThreadDatas<'a>) {
        thread_data
            .column
            .iter_mut()
            .enumerate()
            .for_each(|(y, pixel)| P::process_pixel(thread_data.x, y, pixel))
    }
}

pub struct ByChunkThreadStrategy {
    pub chunk_width: usize,
    pub chunk_height: usize,
}

impl ProcessingStrategy for ByChunkThreadStrategy {
    type ThreadDatas<'a> = ByChunkThreadData<'a>;

    fn thread_datas<'a>(
        &self,
        output_array: &'a mut ImageArray,
    ) -> impl Iterator<Item = Self::ThreadDatas<'a>> {
        // Take the first element from each slice (the column)
        let make_chunk = move |rows: &mut Vec<&'a mut [Pixel]>| {
            rows.iter_mut()
                .try_fold(Vec::new(), move |mut chunk_rows, row| {
                    // Try to remove the first `self.chunk_width` elements of
                    // the row, and add this to the chunk to be processed.
                    // If the row is empty, then we assume that all the pixels
                    // in the column are empty and terminate early.
                    let chunk_row = row.split_off_mut(..self.chunk_width)?;
                    chunk_rows.push(chunk_row);
                    Some(chunk_rows)
                })
        };
        let rows_to_thread_datas = move |y: usize, mut rows: Vec<_>| {
            repeat_with(move || make_chunk(&mut rows))
                .flatten()
                .enumerate()
                .map(|(i, chunk_rows)| (i * self.chunk_width, chunk_rows))
                .map(move |(x, chunk_rows)| ByChunkThreadData {
                    x,
                    y,
                    up_to_down: false,
                    chunk_rows,
                })
        };

        output_array
            .0
            .as_mut_slice()
            .chunks_mut(self.chunk_height)
            .map(to_vec_of_mutable_rows)
            .enumerate()
            .map(|(i, rows)| (i * self.chunk_height, rows))
            .flat_map(move |(y, rows)| rows_to_thread_datas(y, rows))
    }

    fn process_data<'a, P: PictureEffect>(mut thread_data: Self::ThreadDatas<'a>) {
        todo!()
    }
}
