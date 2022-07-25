# pixel-art
rust cli to generate pixel art from images



## TODO

- parallelize pixel calculations?
- cleanup logic in `main.rs` into a separate library function
- cleanup parameters like palette size, color space partitions, etc. into CLI params
- visual regression testing against test images under `images/` would be cool


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
