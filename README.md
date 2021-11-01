# safe_join

Use `SafeJoin::safe_join` in place of [`Path::join`] to help prevent directory traversal
attacks.

A call of the form `dir.safe_join(path)` returns and error if any prefix of `path` refers to a
directory outside of `dir`.

Example:
```rust
assert!(home_dir.join("Documents").safe_join("../.bash_logout").is_err());
```

### Detailed explanation

`safe_join` tries to provide the following guarantee:
```rust
dir.safe_join(path).is_ok()
```
if-and-only-if, for every prefix `prefix` of `path`,
```rust
normalize(paternalize_n_x(dir.join(prefix))).starts_with(normalize(paternalize_n_x(dir)))
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

\* [`path_clean::PathClean::clean`] uses strings internally, so it only works with UTF-8 paths.

### Limitations

**`safe_join` does not consult the filesystem.** So, for example, in a call of the form
`dir.safe_join(path)`, `safe_join` would not consider:

* whether `dir` is a directory
* whether `path` refers to a symlink

There are a host of problems that come with consulting the filesystem. For example, a
programmer might construct a path for a filesystem that is not yet mounted. We want `safe_join`
to be applicable in such situations. So we have chosen to adopt a simple semantics that
considers only a path's [components].

A similar crate that *does* consult the filesystem is [`canonical_path`].

### Camino

`safe_join` optionally supports [`camino::Utf8Path`]. To take advantage of this feature, enable
it on your `Cargo.toml` file's `safe_join` dependency:
```toml
safe_join = { version = "0.1", features = ["camino"] }
```

### Linting

The `safe_join` repository includes a [Dylint] library to check for:

* calls to [`Path::join`] where `SafeJoin::safe_join` could be used
* calls to `SafeJoin::safe_join` that are likely erroneous because they return an error under
  normal circumstances (e.g., `safe_join("..")`)

To use the library:

* Install `cargo-dylint` and `dylint-link` as described in the Dylint [README]:
  ```sh
  cargo install cargo-dylint dylint-link
  ```
* Add the following to your workspace's `Cargo.toml` file:
  ```toml
  [workspace.metadata.dylint]
  libraries = [
      { git = "https://github.com/trailofbits/safe_join", pattern = "lint" },
  ]
  ```
* Run `cargo-dylint` from your workspace's root directory:
  ```sh
  cargo dylint safe_join_lint --workspace
  ```

### References

* [Reddit: Anyone knows how to `fs::canonicalize`, but without actually checking that file exists?](https://www.reddit.com/r/rust/comments/hkkquy/anyone_knows_how_to_fscanonicalize_but_without/)
* [rust-lang/rust: `Path::join` should concat paths even if the second path is absolute #16507](https://github.com/rust-lang/rust/issues/16507)
* [Stack Overflow: Getting the absolute path from a `PathBuf`](https://stackoverflow.com/questions/30511331/getting-the-absolute-path-from-a-pathbuf)

[`camino::Utf8Path`]: https://docs.rs/camino/1.0.5/camino/struct.Utf8Path.html
[`canonical_path`]: https://docs.rs/canonical_path
[`cargo_util::paths::normalize_path`]: https://docs.rs/cargo-util/0.1.1/cargo_util/paths/fn.normalize_path.html
[components]: https://doc.rust-lang.org/std/path/enum.Component.html
[Dylint]: https://github.com/trailofbits/dylint
[`Path::join`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.join
[`lexiclean::Lexiclean::lexiclean`]: https://docs.rs/lexiclean/0.0.1/lexiclean/trait.Lexiclean.html#tymethod.lexiclean
[`path_clean::PathClean::clean`]: https://docs.rs/path-clean/0.1.0/path_clean/trait.PathClean.html#tymethod.clean
[README]: https://github.com/trailofbits/dylint/blob/master/README.md
[`safe_join` lints]: #linting

License: MIT OR Apache-2.0
