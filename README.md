# safe_path

Use `SafePath::safe_join` and `SafePath::safe_parent` in place of [`Path::join`] and
[`Path::parent`] (respectively) to help prevent directory traversal attacks:

* A call of the form `dir.safe_join(path)` returns an error if, for some prefix `prefix` of
  `path`, `dir.join(prefix)` refers to a file outside of `dir`, or if `dir.join(path)` is `dir`.
* A call of the form `dir.safe_parent()` returns an error if `dir.parent()` refers to a file
  inside of `dir`, or if `dir.parent()` is `Some(dir)`.

Examples:
```rust
assert!(home_dir.join("Documents").safe_join("../.bash_logout").is_err());
assert!(home_dir.join("Documents").join("..").safe_parent().is_err());
```

`SafePath::relaxed_safe_join` and `SafePath::relaxed_safe_parent` are variants of
`SafePath::safe_join` and `SafePath::safe_parent` (respectively) that drop the requirement
that the result is not `self`:
```rust
assert!(home_dir.join("Documents").safe_join(".").is_err());
assert!(home_dir.join("Documents").relaxed_safe_join(".").is_ok());

assert!(Path::new("/").safe_parent().is_err());
assert!(Path::new("/").relaxed_safe_parent().is_ok());
```

### Detailed explanation

We'll explain `relaxed_safe_join` in detail since its requirements are slightly simpler than
those of `safe_join`.

`relaxed_safe_join` tries to provide the following guarantee:
```rust
dir.relaxed_safe_join(path).is_ok()
```
if-and-only-if, for every prefix `prefix` of `path`,
```rust
normalize(paternalize_n_x(dir.join(prefix)))
    .starts_with(
        normalize(paternalize_n_x(dir))
    )
```
where the `paternalize_n_x` and `normalize` functions are as follows.

Let *n* be the total number of components in both `dir` and `path`. (Why this choice of *n*?
Because this is an upper bound on the number of parent directories that `dir.join(path)`
could possibly escape.)

Let *x* be any normal component that does not appear in either `dir` or `path`.

A call of the form `paternalize_n_x(path)`:
* prepends *n* copies of *x* to `path`, if `path` is relative
* returns `path` as-is, if `path` is absolute

For example, suppose `dir` is `./w` and `path` is `y/../../z`. Then *n* is 6. Furthermore, `x`
is a normal component not in `dir` or `path`. So `paternalize_n_x(dir)` and
`paternalize_n_x(dir.join(path))` could be as follows:
* `paternalize_n_x(dir) = x/x/x/x/x/x/./w`
* `paternalize_n_x(dir.join(path)) = x/x/x/x/x/x/./w/y/../../z`

There are several path normalization functions implemented in Rust. The ones that we know about
are listed below. To the best of our knowledge, the above guarantee holds using any one of them
as the `normalize` function.
* [`cargo_util::paths::normalize_path`]
* [`lexiclean::Lexiclean::lexiclean`]
* [`path_clean::PathClean::clean`]\*

The guarantee that `relaxed_safe_parent` tries to provide is similar:
```rust
dir.relaxed_safe_parent().is_ok()
```
if-and-only-if
```rust
match dir.parent() {
    None => true,
    Some(parent) => {
        normalize(paternalize_m_x(dir))
            .starts_with(
                normalize(paternalize_m_x(parent))
            )
    }
}
```
where *m* is the total number of components in `dir`.

### Limitations

**`safe_path` does not consult the filesystem.** So, for example, in a call of the form
`dir.safe_join(path)`, `safe_join` would not consider:

* whether `dir` is a directory
* whether `path` refers to a symlink

There are a host of problems that come with consulting the filesystem. For example, a
programmer might construct a path for a filesystem that is not yet mounted. We want `safe_path`
to be applicable in such situations. So we have chosen to adopt a simple semantics that
considers only a path's [components].

A similar crate that *does* consult the filesystem is [`canonical_path`].

### Performance

Benchmarks suggest that `SafePath::safe_join` is about 3.5 times as slow as [`Path::join`],
and that `SafePath::safe_parent` is about 5 times as slow as [`Path::parent`].

However, benchmarks also suggest that normalizing and comparing [`Path::join`]'s `self` and
result using the fastest of the above normalization functions (`normalize_path`) is about 1.5
times slower still. Similarly, normalizing and comparing [`Path::parent`]'s `self` and result is
about 1.2 times slower.

So while using `SafePath::safe_join`/`SafePath::safe_parent` will cause a one to incur some
slowdown, it seems to be less that what one would incur by implementing the same checks
manually.

### Camino

`safe_path` optionally supports [`camino::Utf8Path`]. To take advantage of this feature, enable
it on your `Cargo.toml` file's `safe_path` dependency:
```toml
safe_path = { version = "0.1", features = ["camino"] }
```

### Linting

The `safe_path` repository includes a [Dylint] library to check for:

* calls to [`Path::join`] where `SafePath::safe_join` or `SafePath::relaxed_safe_join` could be
  used
* calls to [`Path::parent`] where `SafePath::safe_parent` or `SafePath::relaxed_safe_parent` could
  be used
* calls to `SafePath::safe_join`/`SafePath::relaxed_safe_join` that are likely erroneous because
  they return an error under normal circumstances, e.g., `safe_join("..")`

To use the library:

* Install `cargo-dylint` and `dylint-link` as described in the Dylint [README]:
  ```sh
  cargo install cargo-dylint dylint-link
  ```
* Add the following to your workspace's `Cargo.toml` file:
  ```toml
  [workspace.metadata.dylint]
  libraries = [
      { git = "https://github.com/trailofbits/safe_path", pattern = "lint" },
  ]
  ```
* Run `cargo-dylint` from your workspace's root directory:
  ```sh
  cargo dylint safe_path_lint --workspace
  ```

### References

* [Reddit: Anyone knows how to `fs::canonicalize`, but without actually checking that file exists?](https://www.reddit.com/r/rust/comments/hkkquy/anyone_knows_how_to_fscanonicalize_but_without/)
* [rust-lang/rust: `Path::join` should concat paths even if the second path is absolute #16507](https://github.com/rust-lang/rust/issues/16507)
* [rust-lang/rust: parent() returns Some("") for single-component relative paths #36861](https://github.com/rust-lang/rust/issues/36861)
* [Stack Overflow: Getting the absolute path from a `PathBuf`](https://stackoverflow.com/questions/30511331/getting-the-absolute-path-from-a-pathbuf)

### Notes

\* [`path_clean::PathClean::clean`] uses strings internally, so it only works with UTF-8 paths.

[`camino::Utf8Path`]: https://docs.rs/camino/1.0.5/camino/struct.Utf8Path.html
[`canonical_path`]: https://docs.rs/canonical_path
[`cargo_util::paths::normalize_path`]: https://docs.rs/cargo-util/0.1.1/cargo_util/paths/fn.normalize_path.html
[components]: https://doc.rust-lang.org/std/path/enum.Component.html
[Dylint]: https://github.com/trailofbits/dylint
[`Path::join`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.join
[`Path::parent`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.parent
[`lexiclean::Lexiclean::lexiclean`]: https://docs.rs/lexiclean/0.0.1/lexiclean/trait.Lexiclean.html#tymethod.lexiclean
[`path_clean::PathClean::clean`]: https://docs.rs/path-clean/0.1.0/path_clean/trait.PathClean.html#tymethod.clean
[README]: https://github.com/trailofbits/dylint/blob/master/README.md

License: MIT OR Apache-2.0
