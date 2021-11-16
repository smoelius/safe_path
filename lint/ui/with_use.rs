use anyhow::Result;
use safe_path::SafePath;
use std::path::{Path, PathBuf};

const ROOT_DIR: &str = "/";
const CUR_DIR: &str = ".";
const PARENT_DIR: &str = "..";
const NORMAL: &str = "y";

fn main() {
    let dir = Path::new("x");
    let path = Path::new(".").join("y");

    let _ = dir.join(path);

    let _ = dir.parent();

    let _ = dir.safe_join("/");
    let _ = dir.safe_join(".");
    let _ = dir.safe_join("..");
    let _ = dir.safe_join("y");

    let _ = dir.safe_join(ROOT_DIR);
    let _ = dir.safe_join(CUR_DIR);
    let _ = dir.safe_join(PARENT_DIR);
    let _ = dir.safe_join(NORMAL);

    let _ = dir.relaxed_safe_join("/");
    let _ = dir.relaxed_safe_join(".");
    let _ = dir.relaxed_safe_join("..");
    let _ = dir.relaxed_safe_join("y");

    let _ = dir.relaxed_safe_join(ROOT_DIR);
    let _ = dir.relaxed_safe_join(CUR_DIR);
    let _ = dir.relaxed_safe_join(PARENT_DIR);
    let _ = dir.relaxed_safe_join(NORMAL);

    let _ = foo().unwrap();
    let _ = bar().unwrap();
    let _ = baz().unwrap();
    let _ = qux().unwrap();
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

fn baz() -> Result<Option<PathBuf>> {
    let dir = Path::new("x");

    Ok(dir.parent().map(Path::to_path_buf))
}

fn qux() -> std::result::Result<Option<PathBuf>, String> {
    let dir = Path::new("x");

    Ok(dir.parent().map(Path::to_path_buf))
}
