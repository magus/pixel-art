# pixel-art
rust cli to generate pixel art from images



## TODO

- profile performance, can we make it faster?
- parallelize pixel calculations?
- cleanup logic in `main.rs` into a separate library function
- cleanup parameters like palette size, color space partitions, etc. into CLI params
- visual regression testing against test images under `images/` would be cool


## Profiling

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
