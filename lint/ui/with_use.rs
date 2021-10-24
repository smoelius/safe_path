use anyhow::Result;
use safe_join::SafeJoin;
use std::path::{Path, PathBuf};

const ROOT_DIR: &str = "/";
const CUR_DIR: &str = ".";
const PARENT_DIR: &str = "..";
const NORMAL: &str = "y";

fn main() {
    let dir = Path::new("x");
    let path = Path::new(".").join("y");

    let _ = dir.join(path);

    let _ = foo().unwrap();

    let _ = bar().unwrap();

    let _ = dir.try_safe_join("/");
    let _ = dir.try_safe_join(".");
    let _ = dir.try_safe_join("..");
    let _ = dir.try_safe_join("y");

    let _ = dir.try_safe_join(ROOT_DIR);
    let _ = dir.try_safe_join(CUR_DIR);
    let _ = dir.try_safe_join(PARENT_DIR);
    let _ = dir.try_safe_join(NORMAL);

    let _ = dir.safe_join("/");
    let _ = dir.safe_join(".");
    let _ = dir.safe_join("..");
    let _ = dir.safe_join("y");

    let _ = dir.safe_join(ROOT_DIR);
    let _ = dir.safe_join(CUR_DIR);
    let _ = dir.safe_join(PARENT_DIR);
    let _ = dir.safe_join(NORMAL);
}

fn foo() -> Result<PathBuf> {
    let dir = Path::new("x");
    let path = Path::new(".").join("y");

    Ok(dir.join(path).to_path_buf())
}

fn bar() -> std::result::Result<PathBuf, String> {
    let dir = Path::new("x");
    let path = Path::new(".").join("y");

    Ok(dir.join(path).to_path_buf())
}
