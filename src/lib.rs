//! Use [`SafeJoin::safe_join`] in place of [`Path::join`] to help prevent directory traversal
//! attacks.
//!
//! A call of the form `dir.safe_join(path)` returns and error if any prefix of `path` refers to a
//! directory outside of `dir`.
//!
//! Example:
//! ```
//! # use safe_join::SafeJoin;
//! # let home_dir = std::path::PathBuf::new();
//! assert!(home_dir.join("Documents").safe_join("../.bash_logout").is_err());
//! ```
//!
//! ## Detailed explanation
//!
//! `safe_join` tries to provide the following guarantee:
//! ```
//! # use safe_join::SafeJoin;
//! # let dir = std::path::PathBuf::new();
//! # let path = std::path::PathBuf::new();
//! dir.safe_join(path).is_ok()
//! # ;
//! ```
//! if-and-only-if, for every prefix `prefix` of `path`,
//! ```
//! # fn normalize(path: std::path::PathBuf) -> std::path::PathBuf { path }
//! # fn paternalize_n_x(path: std::path::PathBuf) -> std::path::PathBuf { path }
//! # let dir = std::path::PathBuf::new();
//! # let prefix = std::path::PathBuf::new();
//! normalize(paternalize_n_x(dir.join(prefix))).starts_with(normalize(paternalize_n_x(dir)))
//! # ;
//! ```
//! where the `paternalize_n_x` and `normalize` functions are as follows.
//!
//! Let *n* be the total number of components in both `dir` and `path`. (Why this choice of *n*?
//! Because this is an upper bound on the number of parent directories that `dir.join(path)`
//! could possibly escape.)
//!
//! Let *x* be any normal component that does not appear in either `dir` or `path`.
//!
//! A call of the form `paternalize_n_x(path)`:
//! * prepends *n* copies of *x* to `path`, if `path` is relative
//! * returns `path` as-is, if `path` is absolute
//!
//! For example, suppose `dir` is `./w` and `path` is `y/../../z`. Then *n* is 6. Furthermore, `x`
//! is a normal component not in `dir` or `path`. So `paternalize_n_x(dir)` and
//! `paternalize_n_x(dir.join(path))` could be as follows:
//! * `paternalize_n_x(dir) = x/x/x/x/x/x/./w`
//! * `paternalize_n_x(dir.join(path)) = x/x/x/x/x/x/./w/y/../../z`
//!
//! There are several path normalization functions implemented in Rust. The ones that we know about
//! are listed below. To the best of our knowledge, the above guarantee holds using any one of them
//! as the `normalize` function.
//! * [`cargo_util::paths::normalize_path`]
//! * [`lexiclean::Lexiclean::lexiclean`]
//! * [`path_clean::PathClean::clean`]\*
//!
//! \* [`path_clean::PathClean::clean`] uses strings internally, so it only works with UTF-8 paths.
//!
//! ## Limitations
//!
//! **`safe_join` does not consult the filesystem.** So, for example, in a call of the form
//! `dir.safe_join(path)`, `safe_join` would not consider:
//!
//! * whether `dir` is a directory
//! * whether `path` refers to a symlink
//!
//! There are a host of problems that come with consulting the filesystem. For example, a
//! programmer might construct a path for a filesystem that is not yet mounted. We want `safe_join`
//! to be applicable in such situations. So we have chosen to adopt a simple semantics that
//! considers only a path's [components].
//!
//! A similar crate that *does* consult the filesystem is [`canonical_path`].
//!
//! ## Camino
//!
//! `safe_join` optionally supports [`camino::Utf8Path`]. To take advantage of this feature, enable
//! it on your `Cargo.toml` file's `safe_join` dependency:
//! ```toml
//! safe_join = { version = "0.1", features = ["camino"] }
//! ```
//!
//! ## Linting
//!
//! The `safe_join` repository includes a [Dylint] library to check for:
//!
//! * calls to [`Path::join`] where [`SafeJoin::safe_join`] could be used
//! * calls to [`SafeJoin::safe_join`] that are likely erroneous because they return an error under
//!   normal circumstances (e.g., `safe_join("..")`)
//!
//! To use the library:
//!
//! * Install `cargo-dylint` and `dylint-link` as described in the Dylint [README]:
//!   ```sh
//!   cargo install cargo-dylint dylint-link
//!   ```
//! * Add the following to your workspace's `Cargo.toml` file:
//!   ```toml
//!   [workspace.metadata.dylint]
//!   libraries = [
//!       { git = "https://github.com/trailofbits/safe_join", pattern = "lint" },
//!   ]
//!   ```
//! * Run `cargo-dylint` from your workspace's root directory:
//!   ```sh
//!   cargo dylint safe_join_lint --workspace
//!   ```
//!
//! ## References
//!
//! * [Reddit: Anyone knows how to `fs::canonicalize`, but without actually checking that file exists?](https://www.reddit.com/r/rust/comments/hkkquy/anyone_knows_how_to_fscanonicalize_but_without/)
//! * [rust-lang/rust: `Path::join` should concat paths even if the second path is absolute #16507](https://github.com/rust-lang/rust/issues/16507)
//! * [Stack Overflow: Getting the absolute path from a `PathBuf`](https://stackoverflow.com/questions/30511331/getting-the-absolute-path-from-a-pathbuf)
//!
//! [`camino::Utf8Path`]: https://docs.rs/camino/1.0.5/camino/struct.Utf8Path.html
//! [`canonical_path`]: https://docs.rs/canonical_path
//! [`cargo_util::paths::normalize_path`]: https://docs.rs/cargo-util/0.1.1/cargo_util/paths/fn.normalize_path.html
//! [components]: std::path::Component
//! [Dylint]: https://github.com/trailofbits/dylint
//! [`Path::join`]: std::path::Path::join
//! [`lexiclean::Lexiclean::lexiclean`]: https://docs.rs/lexiclean/0.0.1/lexiclean/trait.Lexiclean.html#tymethod.lexiclean
//! [`path_clean::PathClean::clean`]: https://docs.rs/path-clean/0.1.0/path_clean/trait.PathClean.html#tymethod.clean
//! [README]: https://github.com/trailofbits/dylint/blob/master/README.md
//! [`safe_join` lints]: #linting

use std::io::{Error, ErrorKind, Result};

/// Abstracts the necessary operations of `std::path::Path` and `camino::Utf8Path`
pub trait PathOps: std::fmt::Debug {
    /// Type returned by [`PathOps::join`] (e.g., [`std::path::PathBuf`])
    type PathBuf: AsRef<Self> + Clone;

    /// Join operation (e.g., [`std::path::Path::join`])
    fn join<P: AsRef<Self>>(&self, path: P) -> Self::PathBuf;

    /// "Starts with" operation (e.g., [`std::path::Path::starts_with`])
    fn starts_with<P: AsRef<Self>>(&self, base: P) -> bool;

    /// Returns true is every prefix of `path` refers to a file within `self`.
    fn is_safe_to_join(&self, path: &Self) -> bool;

    /// Returns true if `self` normalizes to `/`.
    fn is_root(&self) -> bool;
}

/// Trait encapsulating `safe_join`. See [`crate`] documentation for details.
pub trait SafeJoin: PathOps {
    /// Returns `Ok(self.join(path))` if `self` and `path` are safe to join.
    /// # Errors
    /// Returns a [`std::io::Error`] of `kind` [`std::io::ErrorKind::Other`] if any prefix of `path`
    /// refers to a directory outside of `self`. The error payload is unstable and subject to change.
    fn safe_join<P: AsRef<Self>>(&self, path: P) -> Result<Self::PathBuf> {
        if !self.is_safe_to_join(path.as_ref()) {
            return Err(Error::new(
                ErrorKind::Other,
                String::from("unsafe path adjunction"),
            ));
        }
        Ok(self.join(path))
    }
}

impl<P: ?Sized + PathOps> SafeJoin for P {}

macro_rules! is_safe_to_join_body {
    ($self: expr, $path: expr, $ty: path) => {{
        use $ty as Component;
        if $self.is_root() {
            return true;
        }
        let mut n = 0;
        for component in $path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    // smoelius: We know `!$self.is_root()`. Otherwise, we would set `n = 0`.
                    return false;
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if n <= 0 {
                        // smoelius: We know `!$self.is_root()`. Otherwise, we would `continue`.
                        return false;
                    }
                    n -= 1;
                }
                Component::Normal(_) => n += 1,
            }
        }
        true
    }};
}

macro_rules! is_root_body {
    ($self: expr, $ty: path) => {{
        use $ty as Component;
        let mut n: Option<i32> = None;
        for component in $self.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    n = Some(0);
                }
                Component::CurDir => {}
                Component::ParentDir => n = n.map(|n| if n <= 0 { n } else { n - 1 }),
                Component::Normal(_) => n = n.map(|n| n + 1),
            }
        }
        n == Some(0)
    }};
}

impl PathOps for std::path::Path {
    type PathBuf = std::path::PathBuf;
    fn join<P: AsRef<Self>>(&self, path: P) -> Self::PathBuf {
        std::path::Path::join(self, path)
    }
    fn starts_with<P: AsRef<Self>>(&self, base: P) -> bool {
        std::path::Path::starts_with(self, base)
    }
    fn is_safe_to_join(&self, path: &Self) -> bool {
        is_safe_to_join_body!(self, path, std::path::Component)
    }
    fn is_root(&self) -> bool {
        is_root_body!(self, std::path::Component)
    }
}

#[cfg(feature = "camino")]
impl PathOps for camino::Utf8Path {
    type PathBuf = camino::Utf8PathBuf;
    fn join<P: AsRef<Self>>(&self, path: P) -> Self::PathBuf {
        camino::Utf8Path::join(self, path)
    }
    fn starts_with<P: AsRef<Self>>(&self, base: P) -> bool {
        camino::Utf8Path::starts_with(self, base.as_ref())
    }
    fn is_safe_to_join(&self, path: &Self) -> bool {
        is_safe_to_join_body!(self, path, camino::Utf8Component)
    }
    fn is_root(&self) -> bool {
        is_root_body!(self, camino::Utf8Component)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_util::paths::normalize_path;
    use lexiclean::Lexiclean;
    use path_clean::PathClean;
    use std::{
        cmp::max,
        path::{Component, Path, PathBuf},
    };

    fn test<P>(from_str: impl Fn(&'static str) -> P::PathBuf, as_std_path: impl Fn(&P) -> &Path)
    where
        P: ?Sized + PathOps + AsRef<P>,
    {
        let root = from_str("/");
        let cur = from_str(".");
        let parent = from_str("..");
        let normal = from_str("x");
        let dirs = &[
            (true, root.clone()),
            (true, root.as_ref().join(&parent)),
            (true, root.as_ref().join(&normal).as_ref().join(&parent)),
            (false, cur.clone()),
            (false, normal.clone()),
            (false, cur.as_ref().join(&normal)),
            (false, normal.as_ref().join(&cur)),
        ];
        let paths = &[
            (true, cur.clone()),
            (true, normal.clone()),
            (true, cur.as_ref().join(&normal).as_ref().join(&parent)),
            (true, normal.as_ref().join(&cur).as_ref().join(&parent)),
            (true, normal.as_ref().join(&parent).as_ref().join(&cur)),
            (true, normal.as_ref().join(&parent).as_ref().join(&normal)),
            (false, root.clone()),
            (false, parent.clone()),
            (false, normal.as_ref().join(&parent).as_ref().join(&parent)),
            (
                false,
                normal
                    .as_ref()
                    .join(&parent)
                    .as_ref()
                    .join(&parent)
                    .as_ref()
                    .join(&normal),
            ),
        ];
        for (is_root, dir) in dirs {
            assert!(!is_root || dir.as_ref().is_root());
            for (should_succeed_if_dir_is_not_root, path) in paths {
                check_guarantee(
                    *is_root || *should_succeed_if_dir_is_not_root,
                    as_std_path(dir.as_ref()),
                    as_std_path(path.as_ref()),
                );
            }
        }
        check_guarantee(true, as_std_path(root.as_ref()), as_std_path(root.as_ref()));
    }

    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    struct PathBufWrapper(PathBuf);

    impl From<&Path> for PathBufWrapper {
        fn from(path: &Path) -> Self {
            PathBufWrapper(path.to_path_buf())
        }
    }

    impl test_fuzz::Into<&Path> for PathBufWrapper {
        fn into(self) -> &'static Path {
            Box::leak(Box::new(self.0))
        }
    }

    fn fresh_normal(paths: &[&Path]) -> String {
        let n = paths
            .iter()
            .map(|path| path.components())
            .flatten()
            .fold(0, |n, component| {
                if let Component::Normal(s) = component {
                    max(n, s.len())
                } else {
                    n
                }
            });
        format!("{:x>width$}", width = n)
    }

    fn paternalize(n: usize, s: &str, path: &Path) -> PathBuf {
        if path.has_root() {
            path.to_path_buf()
        } else {
            let mut path_buf = PathBuf::new();
            for _ in 0..n {
                path_buf.push(s);
            }
            path_buf.join(path)
        }
    }

    #[test_fuzz::test_fuzz(convert = "&Path, PathBufWrapper")]
    fn check_guarantee(expected: bool, dir: &Path, path: &Path) {
        let normalization_functions: &[(&str, &dyn Fn(&Path) -> PathBuf)] = &[
            ("normalize_path", &normalize_path),
            ("lexiclean", &|path: &Path| Lexiclean::lexiclean(path)),
            ("path_clean", &|path: &Path| {
                PathClean::clean(&path.to_path_buf())
            }),
        ];
        for (name, normalize) in normalization_functions {
            if name == &"path_clean" && (dir.to_str().is_none() || path.to_str().is_none()) {
                continue;
            }

            let n = dir.components().count() + path.components().count();
            let x = fresh_normal(&[dir, path]);

            let np = |path| normalize(&paternalize(n, &x, path));
            let np_dir = np(dir);

            let check = |left: bool, right: bool, prefix: &Path, np_dir_join_prefix: &Path| {
                assert_eq!(
                    left, right,
                    "dir = {:?}, path = {:?}, prefix = {:?}, {}(paternalize(dir)) = {:?}, {}(paternalize(dir.join(prefix))) = {:?}",
                    dir, path, prefix, name, np_dir, name, np_dir_join_prefix,
                );
            };

            let np_dir_join_path = np(&dir.join(path));

            let left = dir.safe_join(path).is_ok();

            #[cfg(not(fuzzing))]
            check(left, expected, &path, &np_dir_join_path);

            let mut right = true;

            for prefix in path.ancestors() {
                let np = |path| normalize(&paternalize(n, &x, path));
                let np_dir_join_prefix = np(&dir.join(prefix));

                right &= np_dir_join_prefix.starts_with(&np_dir);

                if left {
                    check(left, right, &prefix, &np_dir_join_prefix);
                }
            }

            if !left {
                check(left, right, &path, &np_dir_join_path);
            }
        }
    }

    mod std_path {
        use super::*;

        #[test]
        fn test() {
            super::test(PathBuf::from, |path| path);
        }
    }

    #[cfg(feature = "camino")]
    mod camino {
        use ::camino::{Utf8Path, Utf8PathBuf};

        #[test]
        fn test() {
            super::test(Utf8PathBuf::from, Utf8Path::as_std_path);
        }
    }
}
