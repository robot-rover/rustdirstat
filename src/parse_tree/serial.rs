use std::{ffi::OsStr, fs, io, path::Path};

use super::{Dir, Elem, File, FileError, LabelError};

pub fn parse_tree<P: AsRef<Path>>(root: P) -> io::Result<(Dir, Vec<FileError>)> {
    let mut path = root.as_ref().canonicalize()?;
    let name = path.as_os_str().to_owned();

    let mut errors = Vec::new();
    let root_dir = Dir::new(name);
    let root_children = read_dir_entry(&path, |err| errors.push(err));

    let mut dir_stack = vec![(root_dir, root_children)];
    while let Some((dir, children)) = dir_stack.last_mut() {
        if let Some(child) = children.pop() {
            match child {
                Elem::Dir(d) => {
                    path.push::<&OsStr>(&d.name.as_ref());
                    let grand_children = read_dir_entry(&path, |err| errors.push(err));
                    dir_stack.push((d, grand_children));
                }
                Elem::File(f) => {
                    dir.size.files_size += f.size;
                    dir.files.push(f);
                }
                Elem::Other => {}
            }
        } else {
            let (mut dir, _) = dir_stack.pop().unwrap();
            dir.size.total_size += dir.size.files_size;
            path.pop();
            if let Some((parent, _)) = dir_stack.last_mut() {
                parent.size.total_size += dir.size.total_size;
                parent.dirs.push(dir);
            } else {
                return Ok((dir, errors));
            }
        }
    }

    todo!()
}

fn read_dir_entry<F: FnMut(FileError)>(path: &Path, mut err_collect: F) -> Vec<Elem> {
    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(err) => {
            err_collect(err.label(path));
            return Vec::new();
        }
    };
    read_dir
        .map(|res| {
            res.map_err(|err| err.label(path)).and_then(|entry| {
                Ok(match entry.file_type().map_err(|err| err.label(path))? {
                    ft if ft.is_dir() => Elem::Dir(Dir::new(entry.file_name())),
                    ft if ft.is_file() || ft.is_symlink() => Elem::File(File::new(
                        entry.file_name(),
                        entry
                            .metadata()
                            .map_err(|err| err.take_label(path.to_owned().join(entry.file_name())))?
                            .len(),
                    )),
                    // Probably a device file, fifo, etc.
                    // doesn't take up space on the disk
                    _ => Elem::Other,
                })
            })
        })
        .filter_map(|res| res.map_err(&mut err_collect).ok())
        .collect()
}
