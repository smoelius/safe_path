use cargo_util::paths::normalize_path;
use lexiclean::Lexiclean;
use path_clean::PathClean;
use std::{
    cmp::max,
    path::{Component, Path, PathBuf},
};

#[cfg_attr(
    feature = "fuzz",
    derive(Clone, Debug, serde::Deserialize, serde::Serialize)
)]
pub struct PathBufWrapper(PathBuf);

impl From<&Path> for PathBufWrapper {
    fn from(path: &Path) -> Self {
        PathBufWrapper(path.to_path_buf())
    }
}

#[cfg(feature = "fuzz")]
impl test_fuzz::Into<&Path> for PathBufWrapper {
    fn into(self) -> &'static Path {
        Box::leak(Box::new(self.0))
    }
}

pub fn fresh_normal(paths: &[&Path]) -> String {
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
    format!("{:x>width$}", "", width = n + 1)
}

pub fn adopt(n: usize, x: &str, path: &Path) -> PathBuf {
    if path.has_root() {
        path.to_path_buf()
    } else {
        let mut path_buf = PathBuf::new();
        for _ in 0..n {
            path_buf.push(x);
        }
        path_buf.join(path)
    }
}

pub const NORMALIZATION_FUNCTIONS: &[(&str, &dyn Fn(&Path) -> PathBuf)] = &[
    ("normalize_path", &normalize_path),
    ("lexiclean", &|path: &Path| Lexiclean::lexiclean(path)),
    ("path_clean", &|path: &Path| {
        PathClean::clean(&path.to_path_buf())
    }),
];

#[cfg(nightly)]
pub fn normalize_and_compare<P>(
    i: usize,
    as_std_path: impl Fn(&P) -> &Path,
    left: &P,
    right: &P,
) -> bool
where
    P: ?Sized + AsRef<P>,
{
    let left = NORMALIZATION_FUNCTIONS[i].1(as_std_path(left));
    let right = NORMALIZATION_FUNCTIONS[i].1(as_std_path(right));
    left == right
}
