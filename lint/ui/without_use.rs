use std::path::Path;

fn main() {
    let dir = Path::new("x");
    let path = Path::new(".").join("y");

    let _ = dir.join(path);
}
