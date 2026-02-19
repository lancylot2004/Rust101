use clap::ValueEnum;

pub mod invert;

#[derive(Clone, Copy, ValueEnum)]
pub enum Implementation {
    Basic,
    Concurrent,
}
