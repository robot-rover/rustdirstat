use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use super::{Dir, Elem, File, FileError, LabelError};

pub fn parse_tree<P: AsRef<Path>>(root: P) -> io::Result<(Dir, Vec<FileError>)> {
    let mut root_can = root.as_ref().canonicalize()?;
    let name = root_can.as_os_str().to_owned();

    let mut errors = Vec::new();
    let mut root_dir = Dir::new(name);
    let root_children = read_dir_entry(&root_can, &mut |err| errors.push(err));
    recurse_dir(&mut root_dir, &mut root_can, root_children, &mut |err| errors.push(err));
    Ok((root_dir, errors))
}

fn read_dir_entry<P: AsRef<Path>, F: FnMut(FileError)>(path: P, err_collect: &mut F) -> Vec<Elem> {
    let path = path.as_ref();
    match fs::read_dir(path) {
        Ok(walk_dir) => walk_dir.map(|res| {
            res.and_then(|entry| {
                let entry_type = entry.file_type()?;
                if entry_type.is_dir() {
                    Ok(Elem::Dir(Dir::new(entry.file_name())))
                } else if entry_type.is_file() || entry_type.is_symlink() {
                    let meta = entry.metadata()?;
                    Ok(Elem::File(File::new(entry.file_name(), meta.len())))
                } else {
                    // Probably a device file, fifo, etc.
                    // doesn't take up space on the disk
                    Ok(Elem::Other)
                }
            })
        })
        .filter_map(|res| res.map_err(|err| err_collect(err.label(path.into()))).ok())
        .collect(),
        Err(err) => {
            err_collect(err.label(path.into()));
            Vec::new()
        }
    }
}


pub fn recurse_dir<F: FnMut(FileError)>(dir: &mut Dir, path: &mut PathBuf, children: Vec<Elem>, err_collect: &mut F) {
    for elem in children {
        match elem {
            Elem::File(f) => dir.files.push(f),
            Elem::Dir(mut d) => {
                path.push::<&OsStr>(&d.name.as_ref());
                let grand_children = read_dir_entry(path.as_path(),  err_collect);
                recurse_dir(&mut d, path, grand_children, err_collect);
                path.pop();
                dir.dirs.push(d)
            },
            Elem::Other => {},
        }
    }
}
