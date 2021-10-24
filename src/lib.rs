//! Use [`SafeJoin::safe_join`] in place of [`Path::join`] to help prevent directory traversal
//! attacks.
//!
//! A call of the form `dir.safe_join(path)` will panic if any prefix of `path` refers to a
//! directory outside of `dir`.
//!
//! Example:
//! ```should_panic
//! # use safe_join::SafeJoin;
//! # let home_dir = std::path::PathBuf::new();
//! let document = home_dir.join("Documents").safe_join("../.bash_logout"); // panics
//! ```
//! For situations where panicking is not appropriate, there is also [`SafeJoin::try_safe_join`],
//! which returns an [`io::Result`]:
//! ```
//! # use safe_join::SafeJoin;
//! # let home_dir = std::path::PathBuf::new();
//! assert!(home_dir.join("Documents").try_safe_join("../.bash_logout").is_err());
//! ```
//!
//! ## Is it okay that [`SafeJoin::safe_join`] panics?
//!
//! Using [`SafeJoin::safe_join`] in place of [`Path::join`] turns a potential directory traversal
//! vulnerability into a denial-of-service vulnerability. While neither is desirable, the risks
//! associated with the latter are often less. One can switch to [`SafeJoin::safe_join`] to gain
//! immediate protection from directory traversal attacks, and migrate to
//! [`SafeJoin::try_safe_join`] over time.
//!
//! In some rare situations, using [`SafeJoin::safe_join`] in place of [`Path::join`] can introduce
//! a denial-of-service vulnerability. Please consider replacement opportunities individually. One
//! of the [`safe_join` lints] can help to spot cases where [`SafeJoin::safe_join`] has been
//! misapplied.
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
//! * calls to [`Path::join`] where [`SafeJoin::safe_join`]/[`SafeJoin::try_safe_join`] could be
//!   used
//! * calls to [`SafeJoin::safe_join`]/[`SafeJoin::try_safe_join`] where they should *not* be used
//!   because their arguments would cause them to necessarily panic/return an error (e.g.,
//!   `safe_join("..")`)
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
//! [`camino::Utf8Path`]: https://docs.rs/camino/1.0.5/camino/struct.Utf8Path.html
//! [components]: std::path::Component
//! [Dylint]: https://github.com/trailofbits/dylint
//! [`io::Result`]: std::io::Result
//! [`Path::join`]: std::path::Path::join
//! [README]: https://github.com/trailofbits/dylint/blob/master/README.md
//! [`safe_join` lints]: #linting

use std::{
    io::{Error, ErrorKind, Result},
    panic,
};

/// Abstracts the necessary operations of `std::path::Path` and `camino::Utf8Path`
pub trait PathOps: std::fmt::Debug {
    /// Type returned by [`PathOps::join`] (e.g., [`std::path::PathBuf`])
    type PathBuf: AsRef<Self>;

    /// Join operation (e.g., [`std::path::Path::join`])
    fn join<P: AsRef<Self>>(&self, path: P) -> Self::PathBuf;

    /// Checks whether any prefix of `self` refers to a file outside of `.` (i.e.,
    /// [`std::path::Component::CurDir`]).
    /// # Errors
    /// Returns a [`std::io::Error`] of `kind` [`std::io::ErrorKind::Other`] if the check fails.
    /// The error payload is unstable and subject to change.
    fn check_join_safety(&self) -> Result<()>;
}

/// Trait encapsulating `safe_join` and `try_safe_join`. See [`crate`] documentation for details.
pub trait SafeJoin: PathOps {
    /// Returns `self.join(path)` if `self` and `path` are safe to join.
    /// # Panics
    /// Panics if any prefix of `path` refers to a directory outside of `self`.
    fn safe_join<P: AsRef<Self>>(&self, path: P) -> Self::PathBuf {
        #[allow(clippy::panic)]
        self.try_safe_join(&path)
            .unwrap_or_else(|_| panic!("unsafe join of `{:?}` and `{:?}`", self, path.as_ref()))
    }

    /// Returns `Ok(self.join(path))` if `self` and `path` are safe to join.
    /// # Errors
    /// Returns a [`std::io::Error`] of `kind` [`std::io::ErrorKind::Other`] if any prefix of `path`
    /// refers to a directory outside of `self`. The error payload is unstable and subject to change.
    fn try_safe_join<P: AsRef<Self>>(&self, path: P) -> Result<Self::PathBuf> {
        path.as_ref().check_join_safety()?;
        Ok(self.join(path))
    }
}

impl<P: ?Sized + PathOps> SafeJoin for P {}

// smoelius: It would be nice to have a `ComponentOps` trait with a `contribution` method. One could
// then specify in `PathOps` the type that implements `ComponentOps` (e.g., `std::path::Component`)
// similar to how `PathBuf` is specified in `PathOps` now. But `std::path::Component` has a lifetime
// parameter, and generic associated types (https://github.com/rust-lang/rust/issues/44265) are
// still unstable. So using a macro to define a `contribution` function along with the code that
// uses it seems to be the best option for now.

macro_rules! check_join_safety_body {
    ($self: expr, $ty: path) => {{
        use $ty as Component;
        fn contribution(component: Component<'_>) -> Option<isize> {
            match component {
                Component::Prefix(_) | Component::RootDir => None,
                Component::CurDir => Some(0),
                Component::ParentDir => Some(-1),
                Component::Normal(_) => Some(1),
            }
        }
        let mut n = 0;
        for component in $self.components() {
            match (contribution(component), n) {
                (None, _) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        String::from("invalid path component"),
                    ));
                }
                (Some(k), 0) if k < 0 => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        String::from("unsafe path adjunction"),
                    ));
                }
                (Some(k), _) => n += k,
            }
        }
        Ok(())
    }};
}

impl PathOps for std::path::Path {
    type PathBuf = std::path::PathBuf;
    fn join<P: AsRef<Self>>(&self, path: P) -> Self::PathBuf {
        std::path::Path::join(self, path)
    }
    fn check_join_safety(&self) -> Result<()> {
        check_join_safety_body!(self, std::path::Component)
    }
}

#[cfg(feature = "camino")]
impl PathOps for camino::Utf8Path {
    type PathBuf = camino::Utf8PathBuf;
    fn join<P: AsRef<Self>>(&self, path: P) -> Self::PathBuf {
        camino::Utf8Path::join(self, path)
    }
    fn check_join_safety(&self) -> Result<()> {
        check_join_safety_body!(self, camino::Utf8Component)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_join<P>(new: impl Fn(&str) -> &P)
    where
        P: ?Sized + PathOps,
        str: AsRef<P>,
    {
        let dir = new("x");
        let tests = &[
            dir.try_safe_join("."),
            dir.try_safe_join("y"),
            dir.safe_join(".").as_ref().try_safe_join("y"),
            dir.safe_join("y").as_ref().try_safe_join("."),
            dir.try_safe_join(new(".").join("y").as_ref().join("..")),
            dir.try_safe_join(new("y").join(".").as_ref().join("..")),
            dir.try_safe_join(new("y").join("..").as_ref().join(".")),
            dir.try_safe_join(new("y").join("..").as_ref().join("z")),
        ];
        assert!(tests.iter().all(Result::is_ok));
    }

    fn unsafe_join<P>(new: impl Fn(&str) -> &P)
    where
        P: ?Sized + PathOps,
        str: AsRef<P>,
    {
        let dir = new("x");
        let tests = &[
            dir.try_safe_join(".."),
            dir.safe_join(".").as_ref().try_safe_join(".."),
            dir.try_safe_join(new("y").join("..").as_ref().join("..")),
            dir.try_safe_join(new("y").join("..").as_ref().join("..").as_ref().join("z")),
        ];
        assert!(tests.iter().all(Result::is_err));
    }

    mod std_path {
        #[test]
        fn safe_join() {
            super::safe_join(std::path::Path::new);
        }

        #[test]
        fn unsafe_join() {
            super::unsafe_join(std::path::Path::new);
        }
    }

    #[cfg(feature = "camino")]
    mod camino {
        #[test]
        fn safe_join() {
            super::safe_join(::camino::Utf8Path::new);
        }

        #[test]
        fn unsafe_join() {
            super::unsafe_join(::camino::Utf8Path::new);
        }
    }
}
