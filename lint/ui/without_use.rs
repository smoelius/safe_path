use std::path::{Path, PathBuf};

fn main() {
    let dir = Path::new("x");
    let path = Path::new(".").join("y");

    let _ = dir.join(path);

    let _ = foo();
}

fn foo() -> Option<PathBuf> {
    let dir = Path::new("x");

    dir.parent().map(Path::to_path_buf)
}
