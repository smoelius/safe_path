#![cfg_attr(nightly, feature(bench_black_box, test))]

#[cfg(nightly)]
extern crate test;

use safe_path::{PathOps, SafePath};
use std::path::Path;

mod common;
use common::{adopt, fresh_normal, NORMALIZATION_FUNCTIONS};

fn test_cases<P, F>(
    from_str: F,
) -> (
    P::PathBuf,
    Vec<(bool, P::PathBuf)>,
    Vec<(bool, bool, P::PathBuf)>,
)
where
    P: ?Sized + PathOps + AsRef<P>,
    F: Fn(&'static str) -> P::PathBuf,
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
        (true, false, cur.clone()),
        (true, true, normal.clone()),
        (
            true,
            false,
            cur.as_ref().join(&normal).as_ref().join(&parent),
        ),
        (
            true,
            false,
            normal.as_ref().join(&cur).as_ref().join(&parent),
        ),
        (
            true,
            false,
            normal.as_ref().join(&parent).as_ref().join(&cur),
        ),
        (
            true,
            true,
            normal.as_ref().join(&parent).as_ref().join(&normal),
        ),
        (false, false, root.clone()),
        (false, false, parent.clone()),
        (
            false,
            false,
            normal.as_ref().join(&parent).as_ref().join(&parent),
        ),
        (
            false,
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
    (root, dirs.to_vec(), paths.to_vec())
}

#[cfg_attr(
    feature = "fuzz",
    test_fuzz::test_fuzz(convert = "&Path, common::PathBufWrapper")
)]
fn safe_join_guarantee(expected: bool, relaxed: bool, dir: &Path, path: &Path) {
    for (name, normalize) in NORMALIZATION_FUNCTIONS {
        if name == &"path_clean" && (dir.to_str().is_none() || path.to_str().is_none()) {
            continue;
        }

        let n = dir.components().count() + path.components().count();
        let x = fresh_normal(&[dir, path]);

        let np = |path| normalize(&adopt(n, &x, path));
        let np_dir = np(dir);

        let info = |prefix: &Path, np_dir_join_prefix: &Path| {
            format!(
                "dir = {:?}, path = {:?}, prefix = {:?}, {}(adopt(dir)) = {:?}, {}(adopt(dir.join(prefix))) = {:?}",
                dir, path, prefix, name, np_dir, name, np_dir_join_prefix,
            )
        };

        let np_dir_join_path = np(&dir.join(path));

        let equal = np_dir == np_dir_join_path;

        let left = if relaxed {
            dir.relaxed_safe_join(path)
        } else {
            dir.safe_join(path)
        }
        .is_ok();

        if !relaxed && equal {
            assert!(!left, "{}", info(path, &np_dir_join_path));
            return;
        }

        #[cfg(not(fuzzing))]
        assert_eq!(left, expected, "{}", info(&path, &np_dir_join_path));

        let mut right = true;

        for prefix in path.ancestors() {
            let np = |path| normalize(&adopt(n, &x, path));
            let np_dir_join_prefix = np(&dir.join(prefix));

            right &= np_dir_join_prefix.starts_with(&np_dir);

            if left {
                assert_eq!(left, right, "{}", info(&prefix, &np_dir_join_prefix));
            }
        }

        if !left {
            assert_eq!(left, right, "{}", info(&path, &np_dir_join_path));
        }
    }
}

fn test<P>(from_str: impl Fn(&'static str) -> P::PathBuf, as_std_path: impl Fn(&P) -> &Path)
where
    P: ?Sized + PathOps + AsRef<P>,
{
    let (root, dirs, paths) = test_cases::<P, _>(from_str);
    for (dir_is_root, dir) in dirs {
        // smoelius: Do not remove the next line. It is a sanity check that `is_root` works.
        assert_eq!(dir_is_root, dir.as_ref().is_root());
        for (should_succeed_if_dir_is_not_root, but_only_if_relaxed, path) in &paths {
            assert!(*should_succeed_if_dir_is_not_root || !but_only_if_relaxed);
            safe_join_guarantee(
                *should_succeed_if_dir_is_not_root || dir_is_root,
                !but_only_if_relaxed,
                as_std_path(dir.as_ref()),
                as_std_path(path.as_ref()),
            );
        }
    }
    safe_join_guarantee(
        true,
        true,
        as_std_path(root.as_ref()),
        as_std_path(root.as_ref()),
    );
}

#[cfg(nightly)]
fn bench<P>(
    bencher: &mut test::Bencher,
    from_str: impl Fn(&'static str) -> P::PathBuf,
    join_like_op: impl Fn(&P, &P),
) where
    P: ?Sized + PathOps + AsRef<P>,
{
    let (_, dirs, paths) = test_cases::<P, _>(from_str);
    bencher.iter(|| {
        for (_, dir) in &dirs {
            for (_, _, path) in &paths {
                join_like_op(dir.as_ref(), path.as_ref());
            }
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
            fn a_join(bencher: &mut test::Bencher) {
                bench(bencher, $from_str, |dir: &$path_ty, path: &$path_ty| {
                    let _ = black_box(dir.join(path));
                });
            }

            #[bench]
            fn b_safe_join(bencher: &mut test::Bencher) {
                bench(bencher, $from_str, |dir: &$path_ty, path: &$path_ty| {
                    let _ = black_box(dir.safe_join(path));
                });
            }

            #[bench]
            fn c_normalize_path(bencher: &mut test::Bencher) {
                bench(bencher, $from_str, |dir: &$path_ty, path: &$path_ty| {
                    let _ = black_box(normalize_and_compare(0, $as_std_path, dir, &dir.join(path)));
                });
            }

            #[bench]
            fn d_lexiclean(bencher: &mut test::Bencher) {
                bench(bencher, $from_str, |dir: &$path_ty, path: &$path_ty| {
                    let _ = black_box(normalize_and_compare(1, $as_std_path, dir, &dir.join(path)));
                });
            }

            #[bench]
            fn e_path_clean(bencher: &mut test::Bencher) {
                bench(bencher, $from_str, |dir: &$path_ty, path: &$path_ty| {
                    let _ = black_box(normalize_and_compare(2, $as_std_path, dir, &dir.join(path)));
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
