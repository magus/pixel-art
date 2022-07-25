# pixel-art
rust cli to generate pixel art from images



## TODO

- cleanup logic in `main.rs` into a separate library function
- cleanup parameters like palette size, color space partitions, etc. into CLI params
- visual regression testing against test images under `images/` would be cool


## Notes

### 2022-07-25

Refactoring long loops over pixels to use `rayon` for parallelization
Significant reduction in time spent `calcuate_pixelated_grid_cells`.
The results are drastic in both debug and release builds.

> **Debug**

**3.1x faster** (`29.759s` to `9.57s`)

> Before
```
⏱️ [times::all]
   [reading_image] 2.096882366s
   [zealous_crop] 2.063461401s
   [palette] 2.505528379s
   [initialize_color_counts] 293.264µs
   [create_pixelated_buffer] 7.417µs
   [calcuate_pixelated_grid_cells] 22.726531768s
   [put_pixel_pixelated] 1.877211ms
   [output_pixelated] 3.969701ms
cargo run  31.56s user 0.09s system 106% cpu 29.759 total
```

> After
```
⏱️ [times::all]
   [reading_image] 2.039194323s
   [zealous_crop] 1.98774807s
   [palette] 2.517575879s
   [initialize_color_counts] 306.857µs
   [create_pixelated_buffer] 7.742µs
   [calcuate_pixelated_grid_cells] 2.667941957s
   [put_pixel_pixelated] 2.122397ms
   [output_pixelated] 4.624637ms
cargo run  50.67s user 0.14s system 530% cpu 9.570 total
```

> **Release**

**2.0x faster** (`1.215s` to `0.609s`)

> Before
```
⏱️ [times::all]
   [reading_image] 62.616668ms
   [zealous_crop] 97.17503ms
   [palette] 80.678959ms
   [initialize_color_counts] 111.536µs
   [create_pixelated_buffer] 4.576µs
   [calcuate_pixelated_grid_cells] 689.217486ms
   [put_pixel_pixelated] 55.638µs
   [output_pixelated] 419.743µs
cargo run --release  1.06s user 0.13s system 97% cpu 1.215 total
```

> After
```
⏱️ [times::all]
   [reading_image] 62.574894ms
   [zealous_crop] 96.457635ms
   [palette] 75.510148ms
   [initialize_color_counts] 140.31µs
   [create_pixelated_buffer] 4.274µs
   [calcuate_pixelated_grid_cells] 89.934462ms
   [put_pixel_pixelated] 88.328µs
   [output_pixelated] 449.553µs
cargo run --release  1.78s user 0.14s system 314% cpu 0.609 total
```

## Running

Release mode applies many LLVM optimizations, `cargo run` is a very poor indicator of final performance

> images/panda-bear.JPG
```
> time cargo run
cargo run  32.52s user 0.10s system 106% cpu 30.721 total

> time cargo run --release
cargo run --release  1.05s user 0.13s system 97% cpu 1.202 total
```

### Profiling

```
cargo install flamegraph
flamegraph --root -- target/debug/pixel-art
```


## Help

### `failed to fetch` `no authentication available`

```
error: failed to fetch `https://github.com/rust-lang/crates.io-index`

Caused by:
  failed to authenticate when downloading repository: git@github.com:rust-lang/crates.io-index

  * attempted ssh-agent authentication, but no usernames succeeded: `git`

  if the git CLI succeeds then `net.git-fetch-with-cli` may help here
  https://doc.rust-lang.org/cargo/reference/config.html#netgit-fetch-with-cli

Caused by:
  no authentication available
```

Add your local SSH identity to the `ssh-agent` for use when fetching via `ssh-agent` used by `cargo`

```
eval `ssh-agent -s`
ssh-add
```

> https://github.com/rust-lang/cargo/issues/3381
