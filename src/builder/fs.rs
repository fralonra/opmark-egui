use std::{
    fs::{create_dir, remove_dir_all, remove_file, rename, File},
    io::prelude::*,
    path::Path,
};

pub fn mkdir(dir: &Path) {
    rm(dir);

    create_dir(dir).expect(&format!(
        "[ERROR] failed to create directory `{}`",
        dir.display()
    ));
}

pub fn mkfile(path: &Path, buf: &[u8]) {
    rm(path);

    File::create(path)
        .expect(&format!(
            "[ERROR] failed to create file `{}`",
            path.display()
        ))
        .write_all(buf)
        .expect(&format!(
            "[ERROR] failed to write to file `{}`",
            path.display()
        ));
}

pub fn mvfile(from: &Path, to: &Path) {
    rename(from, to).expect(&format!(
        "[ERROR] failed to rename `{}` to `{}`",
        from.display(),
        to.display()
    ));
}

pub fn rm(path: &Path) {
    if path.is_dir() {
        remove_dir_all(path).expect(&format!(
            "[ERROR] failed to remove directory `{}`",
            path.display()
        ));
    } else if path.is_file() {
        remove_file(path).expect(&format!(
            "[ERROR] failed to remove file `{}`",
            path.display()
        ));
    }
}
