use std::{
    ffi::OsString,
    fs, io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use super::{Dir, Elem, File, FileError, LabelError};

pub fn parse_tree<P: AsRef<Path>>(root: P) -> io::Result<(Dir, Vec<FileError>)> {
    let mut root_can = root.as_ref().canonicalize()?;
    let name = root_can.file_name().unwrap().to_owned();
    let read_dir = fs::read_dir(&root_can)?;

    let mut errors = Vec::new();
    let mut root_dir = Dir::new(name);
    let root_children = read_dir_entry(read_dir, |err| errors.push(err.label(root_can.clone())));
    recurse_dir(&mut root_dir, &mut root_can, root_children, &mut |err| errors.push(err));
    Ok((root_dir, errors))
}

fn read_dir_entry<F: FnMut(io::Error)>(contents: fs::ReadDir, mut err_collect: F) -> Vec<OsString> {
    let entry_list = contents
        .filter_map(|res| res.map_err(&mut err_collect).ok())
        .map(|entry| entry.file_name())
        .collect();
    entry_list
}

pub fn recurse_dir<F: FnMut(FileError)>(dir: &mut Dir, path: &mut PathBuf, mut children: Vec<OsString>, err_collect: &mut F) {
    while !children.is_empty() {
        let child_name = children.pop().unwrap();
        path.push(child_name.as_os_str());
        match read_dir_or_file(child_name, path, err_collect) {
            Ok(Elem::Dir(d)) => dir.dirs.push(d),
            Ok(Elem::File(f)) => dir.files.push(f),
            Ok(Elem::Other) => {}
            Err(err) => err_collect(err.label(path.clone())),
        }
        path.pop();
    }
}

pub fn read_dir_or_file<F: FnMut(FileError)>(child_name: OsString, path: &mut PathBuf, err_collect: &mut F) -> io::Result<Elem> {
    let meta = fs::symlink_metadata(path.as_path())?;
    if meta.is_dir() {
        let read_dir = fs::read_dir(path.as_path())?;
        let mut child_dir = Dir::new(child_name);
        let grand_children = read_dir_entry(read_dir, |err| err_collect(err.label(path.clone())));
        recurse_dir(&mut child_dir, path, grand_children, err_collect);
        Ok(Elem::Dir(child_dir))
    } else if meta.is_file() || meta.is_symlink() {
        let file = File::new(child_name, meta.size());
        Ok(Elem::File(file))
    } else {
        // Probably a device file, fifo, etc.
        // doesn't take up space on the disk
        Ok(Elem::Other)
    }
}
