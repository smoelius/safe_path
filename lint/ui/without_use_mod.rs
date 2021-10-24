fn main() {
    let _ = foo::bar();
}

mod foo {
    use std::path::{Path, PathBuf};

    pub fn bar() -> PathBuf {
        let dir = Path::new("x");
        let path = Path::new(".").join("y");

        dir.join(path).to_path_buf()
    }
}
