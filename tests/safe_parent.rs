#![cfg_attr(nightly, feature(bench_black_box, test))]

#[cfg(nightly)]
extern crate test;

use safe_path::{PathOps, SafePath};
use std::path::{Path, PathBuf};

mod common;
use common::{adopt, fresh_normal, NORMALIZATION_FUNCTIONS};

fn test_cases<P, F>(from_str: F) -> Vec<(bool, bool, P::PathBuf)>
where
    P: ?Sized + PathOps + AsRef<P>,
    F: Fn(&'static str) -> P::PathBuf,
{
    let root = from_str("/");
    let cur = from_str(".");
    let parent = from_str("..");
    let normal = from_str("x");
    let dirs = &[
        (true, true, root.clone()),
        (true, true, root.as_ref().join(&parent)),
        (true, true, cur.clone()),
        (true, false, cur.as_ref().join(&normal)),
        (true, false, normal.clone()),
        (true, false, normal.as_ref().join(&cur)),
        (
            false,
            false,
            root.as_ref().join(&normal).as_ref().join(&parent),
        ),
        (false, false, parent.clone()),
    ];
    dirs.to_vec()
}

#[cfg_attr(
    feature = "fuzz",
    test_fuzz::test_fuzz(convert = "&Path, common::PathBufWrapper")
)]
fn safe_parent_guarantee(expected: bool, relaxed: bool, dir: &Path) {
    for (name, normalize) in NORMALIZATION_FUNCTIONS {
        if name == &"path_clean" && (dir.to_str().is_none()) {
            continue;
        }

        let m = dir.components().count();
        let x = fresh_normal(&[dir]);

        let np = |path| normalize(&adopt(m, &x, path));
        let np_dir = np(dir);

        let info = |np_dir_parent: Option<PathBuf>| {
            let s = np_dir_parent.map_or(String::new(), |np_dir_parent| {
                format!(", {}(adopt(dir.parent())) = {:?}", name, np_dir_parent)
            });
            format!("dir = {:?}, {}(adopt(dir)) = {:?}{}", dir, name, np_dir, s,)
        };

        let (np_dir_parent, equal, right) = match dir.parent() {
            None => (None, dir == Path::new("") || dir.is_root(), true),
            Some(dir_parent) => {
                let np_dir_parent = np(dir_parent);
                let equal = np_dir == np_dir_parent;
                let right = np_dir.starts_with(&np_dir_parent);
                (Some(np_dir_parent), equal, right)
            }
        };

        let left = if relaxed {
            dir.relaxed_safe_parent()
        } else {
            dir.safe_parent()
        }
        .is_ok();

        if !relaxed && equal {
            assert!(!left, "{}", info(np_dir_parent));
            return;
        }

        #[cfg(not(fuzzing))]
        assert_eq!(left, expected, "{}", info(np_dir_parent));

        assert_eq!(left, right, "{}", info(np_dir_parent));
    }
}

fn test<P>(from_str: impl Fn(&'static str) -> P::PathBuf, as_std_path: impl Fn(&P) -> &Path)
where
    P: ?Sized + PathOps + AsRef<P>,
{
    let dirs = test_cases::<P, _>(from_str);
    for (should_succeed, but_only_if_relaxed, dir) in dirs {
        assert!(should_succeed || !but_only_if_relaxed);
        safe_parent_guarantee(
            should_succeed,
            !but_only_if_relaxed,
            as_std_path(dir.as_ref()),
        );
    }
}

#[cfg(nightly)]
fn bench<P>(
    bencher: &mut test::Bencher,
    from_str: impl Fn(&'static str) -> P::PathBuf,
    parent_like_op: impl Fn(&P),
) where
    P: ?Sized + PathOps + AsRef<P>,
{
    let dirs = test_cases::<P, _>(from_str);
    bencher.iter(|| {
        for (_, _, dir) in &dirs {
            parent_like_op(dir.as_ref());
        }
    });
}

macro_rules! mod_body {
    {$path_ty: path, $from_str: expr, $as_std_path: expr} => {
        #[cfg(nightly)]
        use super::*;

        #[test]
        fn test() {
            super::test($from_str, $as_std_path);
        }

        #[cfg(nightly)]
        mod benches {
            use super::{common::normalize_and_compare, *};
            use std::hint::black_box;

            #[bench]
            fn a_parent(bencher: &mut test::Bencher) {
                bench(bencher, $from_str, |dir: &$path_ty| {
                    let _ = black_box(dir.parent());
                });
            }

            #[bench]
            fn b_safe_parent(bencher: &mut test::Bencher) {
                bench(bencher, $from_str, |dir: &$path_ty| {
                    let _ = black_box(dir.safe_parent());
                });
            }

            #[bench]
            fn c_normalize_path(bencher: &mut test::Bencher) {
                bench(bencher, $from_str, |dir: &$path_ty| {
                    let _ = black_box(&dir.parent().map(|parent| normalize_and_compare(0, $as_std_path, dir, parent)));
                });
            }

            #[bench]
            fn d_lexiclean(bencher: &mut test::Bencher) {
                bench(bencher, $from_str, |dir: &$path_ty| {
                    let _ = black_box(&dir.parent().map(|parent| normalize_and_compare(1, $as_std_path, dir, parent)));
                });
            }

            #[bench]
            fn e_path_clean(bencher: &mut test::Bencher) {
                bench(bencher, $from_str, |dir: &$path_ty| {
                    let _ = black_box(&dir.parent().map(|parent| normalize_and_compare(2, $as_std_path, dir, parent)));
                });
            }
        }
    };
}

mod std_path {
    use std::path::PathBuf;

    mod_body! {Path, PathBuf::from, |path| path}
}

#[cfg(feature = "camino")]
mod camino {
    use ::camino::{Utf8Path, Utf8PathBuf};

    mod_body! {Utf8Path, Utf8PathBuf::from, Utf8Path::as_std_path}
}
