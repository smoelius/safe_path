# safe_join

Use `SafeJoin::safe_join` in place of [`Path::join`] to help prevent directory traversal
attacks.

A call of the form `dir.safe_join(path)` will panic if any prefix of `path` refers to a
directory outside of `dir`.

Example:
```rust
let document = home_dir.join("Documents").safe_join("../.bash_logout"); // panics
```
For situations where panicking is not appropriate, there is also `SafeJoin::try_safe_join`,
which returns an [`io::Result`]:
```rust
assert!(home_dir.join("Documents").try_safe_join("../.bash_logout").is_err());
```

### Is it okay that `SafeJoin::safe_join` panics?

Using `SafeJoin::safe_join` in place of [`Path::join`] turns a potential directory traversal
vulnerability into a denial-of-service vulnerability. While neither is desirable, the risks
associated with the latter are often less. One can switch to `SafeJoin::safe_join` to gain
immediate protection from directory traversal attacks, and migrate to
`SafeJoin::try_safe_join` over time.

In some rare situations, using `SafeJoin::safe_join` in place of [`Path::join`] can introduce
a denial-of-service vulnerability. Please consider replacement opportunities individually. One
of the [`safe_join` lints] can help to spot cases where `SafeJoin::safe_join` has been
misapplied.

### Limitations

**`safe_join` does not consult the filesystem.** So, for example, in a call of the form
`dir.safe_join(path)`, `safe_join` would not consider:

* whether `dir` is a directory
* whether `path` refers to a symlink

There are a host of problems that come with consulting the filesystem. For example, a
programmer might construct a path for a filesystem that is not yet mounted. We want `safe_join`
to be applicable in such situations. So we have chosen to adopt a simple semantics that
considers only a path's [components].

### Camino

`safe_join` optionally supports [`camino::Utf8Path`]. To take advantage of this feature, enable
it on your `Cargo.toml` file's `safe_join` dependency:
```toml
safe_join = { version = "0.1", features = ["camino"] }
```

### Linting

The `safe_join` repository includes a [Dylint] library to check for:

* calls to [`Path::join`] where `SafeJoin::safe_join`/`SafeJoin::try_safe_join` could be
  used
* calls to `SafeJoin::safe_join`/`SafeJoin::try_safe_join` where they should *not* be used
  because their arguments would cause them to necessarily panic/return an error (e.g.,
  `safe_join("..")`)

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

[`camino::Utf8Path`]: https://docs.rs/camino/1.0.5/camino/struct.Utf8Path.html
[components]: https://doc.rust-lang.org/std/path/enum.Component.html
[Dylint]: https://github.com/trailofbits/dylint
[`io::Result`]: https://doc.rust-lang.org/std/io/type.Result.html
[`Path::join`]: https://doc.rust-lang.org/std/path/struct.Path.html#method.join
[README]: https://github.com/trailofbits/dylint/blob/master/README.md
[`safe_join` lints]: #linting

License: MIT OR Apache-2.0
