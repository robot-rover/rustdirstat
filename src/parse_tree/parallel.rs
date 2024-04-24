use std::{
    collections::LinkedList,
    ffi::OsStr,
    fs, io,
    path::Path,
};

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use super::{Dir, Elem, File, FileError, FileResult, LabelError};

pub fn parse_tree<P: AsRef<Path>>(root: P) -> io::Result<(Dir, Vec<FileError>)> {
    let root_can = root.as_ref().canonicalize()?;
    let name = root_can.as_os_str().to_owned();

    let mut dir = Dir::new(name);
    let errors = recurse_dir(&mut dir, &root_can.as_path());
    Ok((dir, errors.into_iter().collect()))
}

fn read_dir_entry(path: &Path) -> Vec<FileResult<Elem>> {
    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(err) => return vec![Err(err.label(path))],
    };
    let entry_list = read_dir
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
        .collect();
    entry_list
}

pub fn recurse_dir(dir: &mut Dir, path: &Path) -> LinkedList<FileError> {
    let children: Vec<Result<Elem, FileError>> = read_dir_entry(&path);
    let mut errors = LinkedList::new();
    for child in children {
        match child {
            Ok(Elem::Dir(d)) => dir.dirs.push(d),
            Ok(Elem::File(f)) => {
                dir.size.files_size += f.size;
                dir.files.push(f);
            },
            Ok(Elem::Other) => {}
            Err(err) => errors.push_back(err),
        }
    }
    let mut child_errors = dir
        .dirs
        .par_iter_mut()
        .map(|d| recurse_dir(d, &path.to_owned().join::<&OsStr>(d.name.as_ref())))
        .reduce(
            || LinkedList::new(),
            |mut lhs, mut rhs| {
                lhs.append(&mut rhs);
                lhs
            },
        );

    dir.size.total_size = dir.size.files_size + dir.dirs.iter().map(|d| d.size.total_size).sum::<u64>();
    errors.append(&mut child_errors);
    errors
}
