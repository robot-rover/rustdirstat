use std::{ffi::OsStr, io, path::Path};

use crate::parse_tree::{fs_crossing, read_dir_entry};

use super::{Config, Dir, Elem, FileError, WalkContext};

pub fn parse_tree<P: AsRef<Path>>(root: P, config: Config) -> io::Result<(Dir, Vec<FileError>)> {
    let mut path = root.as_ref().canonicalize()?;
    let name = path.as_os_str().to_owned();

    let context = WalkContext {
        root_fs: if config.same_filesystem {
            fs_crossing::device_num(&path)?
        } else {
            0
        },
        config,
    };

    let mut errors = Vec::new();
    let root_dir = Dir::new(name);
    let root_children = read_dir_entry(&path, &context, |err| errors.push(err));

    let mut dir_stack = vec![(root_dir, root_children)];
    while let Some((dir, children)) = dir_stack.last_mut() {
        if let Some(child) = children.pop() {
            match child {
                Elem::Dir(d) => {
                    path.push::<&OsStr>(&d.name.as_ref());
                    // Err means filesystem boundary crossing.
                    let grand_children = read_dir_entry(&path, &context, |err| errors.push(err));
                    dir_stack.push((d, grand_children));
                }
                Elem::File(f) => {
                    dir.size.files_size += f.size;
                    dir.files.push(f);
                }
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
