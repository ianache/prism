use std::fs;
use std::io;
use std::path::Path;

pub fn write_once(path: &Path, contents: &str) -> io::Result<()> {
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "output already exists",
        ));
    }
    fs::write(path, contents)
}
