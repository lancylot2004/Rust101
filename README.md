# Rust 101

Simple implementations of Game of Life. Uses `clap` and `minifb` for plumbing. These are used to
demonstrate Rust's capabilities in ownership and concurrency.

## Usage

```zsh
Usage: render [OPTIONS] --mode <MODE>

Options:
  -s, --size <SIZE>              Window size in pixels [default: 800x600]
  -m, --mode <MODE>              What strategy to use for stepping the simulation [possible values: serial, parallel, workers, pool]
  -c, --chunk-size <CHUNK_SIZE>  Chunk size. Required when using the [Workers] or [Pool] mode. Ignored otherwise
  -h, --help                     Print help
  -V, --version                  Print version
```

## Implementations

You can find different implementations in `src/implementations`.

- **Serial**: Sequentially iterates over pixels.
- **Parallel**: Splits the buffer into chunks and processes them using threads. Free of
  synchronisation primitives.
- **Workers**: Spawns worker threads that eagerly consume chunks every frame.
- **Pool**: Similar to workers, but uses a thread pool to amortise the cost of spawning threads.
