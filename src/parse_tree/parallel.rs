use std::{
    collections::LinkedList,
    ffi::OsStr, io,
    path::Path,
};

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use super::{fs_crossing, read_dir_entry, Config, Dir, Elem, FileError, WalkContext};

pub fn parse_tree<P: AsRef<Path>>(path: P, config: Config) -> io::Result<(Dir, Vec<FileError>)> {
    let root_can = path.as_ref().to_owned().canonicalize()?;
    let name = root_can.as_os_str().to_owned();

    let context = WalkContext {
        root_fs: if config.same_filesystem { fs_crossing::device_num(&root_can)? } else { 0 },
        config,
    };

    let mut dir = Dir::new(name);
    let errors = recurse_dir(&mut dir, &root_can.as_path(), &context);
    Ok((dir, errors.into_iter().collect()))
}

fn recurse_dir(dir: &mut Dir, path: &Path, context: &WalkContext) -> LinkedList<FileError> {
    let mut errors = LinkedList::new();
    let children = read_dir_entry(&path, context, |err| errors.push_back(err));
    for child in children {
        match child {
            Elem::Dir(d) => dir.dirs.push(d),
            Elem::File(f) => {
                dir.size.files_size += f.size;
                dir.files.push(f);
            },
        }
    }
    let mut child_errors = dir
        .dirs
        .par_iter_mut()
        .map(|d| recurse_dir(d, &path.to_owned().join::<&OsStr>(d.name.as_ref()), context))
        .reduce(
            || LinkedList::new(),
            |mut lhs, mut rhs| {
                lhs.append(&mut rhs);
                lhs
            },
        );

    dir.size.total_size = dir.size.files_size + dir.dirs.iter().map(|d| d.size.total_size).sum::<u64>();
    if errors.len() > 0 {
        child_errors.append(&mut errors);
    }
    child_errors
}
