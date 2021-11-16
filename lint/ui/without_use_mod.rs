fn main() {
    let _ = foo::bar();
    let _ = foo::baz();
}

mod foo {
    use std::path::{Path, PathBuf};

    pub fn bar() -> PathBuf {
        let dir = Path::new("x");
        let path = Path::new(".").join("y");

        dir.join(path).to_path_buf()
    }

    pub fn baz() -> Option<PathBuf> {
        let dir = Path::new("x");

        dir.parent().map(Path::to_path_buf)
    }
}
