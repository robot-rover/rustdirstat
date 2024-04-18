use std::{f32::consts::E, ffi::OsString, fs, io, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

use super::{Dir, Elem, File};

pub fn parse_tree<P: AsRef<Path>>(root: P) -> io::Result<Dir> {
    let mut root_can = root.as_ref().canonicalize()?;
    let name = root_can.file_name().unwrap().to_owned();
    let read_dir = fs::read_dir(&root_can)?;

    let mut root_dir = Dir::new(name, read_dir);

    recurse_dir(&mut root_dir, &mut root_can);
    Ok(root_dir)
}

pub fn recurse_dir(dir: &mut Dir, path: &mut PathBuf) {
    while !dir.unparsed_children.is_empty() {
        let child_name = dir.unparsed_children.pop().unwrap();
        path.push(child_name.as_os_str());
        match read_dir_or_file(child_name, path) {
            Ok(Elem::Dir(d)) => dir.dirs.push(d),
            Ok(Elem::File(f)) => dir.files.push(f),
            Err(err) => dir.errors.push(err),
        }
        path.pop();
    }
}

pub fn read_dir_or_file(child_name: OsString, path: &mut PathBuf) -> io::Result<Elem> {
    let meta = fs::metadata(path.as_path())?;
    if meta.is_dir() {
        let read_dir = fs::read_dir(path.as_path())?;
        let mut child_dir = Dir::new(child_name, read_dir);
        recurse_dir(&mut child_dir, path);
        Ok(Elem::Dir(child_dir))
    } else {
        let file = File::new(child_name, meta.size());
        Ok(Elem::File(file))
    }
}
