use clap::ValueEnum;

pub mod invert;
pub mod rotate;
pub mod blur;

#[derive(Clone, Copy, ValueEnum)]
pub enum Implementation {
    Basic,
    Concurrent,
}
