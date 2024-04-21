use std::{
    ffi::OsStr, fs, io, path::Path
};

use super::{Dir, Elem, File, FileError, LabelError};

pub fn parse_tree<P: AsRef<Path>>(root: P) -> io::Result<(Dir, Vec<FileError>)> {
    let mut path = root.as_ref().canonicalize()?;
    let name = path.as_os_str().to_owned();
    let read_dir = fs::read_dir(&path)?;

    let mut errors = Vec::new();
    let root_dir = Dir::new(name);
    let root_children = read_dir_entry(read_dir, |err| errors.push(err.label(path.clone())));

    let mut dir_stack = vec![(root_dir, root_children)];
    while let Some((dir, children)) = dir_stack.last_mut() {
        if let Some(child) = children.pop() {
            match child {
                Elem::Dir(d) => {
                    path.push::<&OsStr>(&d.name.as_ref());
                    match fs::read_dir(&path) {
                        Ok(contents) => {
                            let grand_children = read_dir_entry(contents, |err| errors.push(err.label(path.clone())));
                            dir_stack.push((d, grand_children));
                        },
                        Err(err) => {
                            errors.push(err.label(path.clone()));
                            path.pop();
                        }
                    }
                },
                Elem::File(f) => dir.files.push(f),
                Elem::Other => {},
            }
        } else {
            let (dir, _) = dir_stack.pop().unwrap();
            path.pop();
            if let Some((parent, _)) = dir_stack.last_mut() {
                parent.dirs.push(dir);
            } else {
                return Ok((dir, errors));
            }
        }
    }

    todo!()
}

fn read_dir_entry<F: FnMut(io::Error)>(contents: fs::ReadDir, mut err_collect: F) -> Vec<Elem> {
    contents
        .map(|res| {
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
        .filter_map(|res| res.map_err(&mut err_collect).ok())
        .collect()
}
