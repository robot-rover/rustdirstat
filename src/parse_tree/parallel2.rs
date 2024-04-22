use std::{
    ffi::OsString,
    fs, io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::{Dir, Elem, File, FileError, LabelError};

pub fn parse_tree<P: AsRef<Path>>(root: P) -> io::Result<(Dir, Vec<FileError>)> {
    let root_can = root.as_ref().canonicalize()?;
    let name = root_can.as_os_str().to_owned();

    if let Elem::Dir(dir) = read_dir_or_file(name, &root_can, rayon::current_num_threads().next_power_of_two() + 1)? {
        Ok((dir, Vec::new()))
    } else {
        panic!("Root is not a directory: {:?}", &root_can)
    }
}

fn read_dir_entry(contents: fs::ReadDir) -> (Vec<OsString>, Vec<io::Error>) {
    let mut errors = Vec::new();
    let entry_list = contents
        .filter_map(|res| match res {
            Ok(entry) => Some(entry.file_name()),
            Err(err) => {
                errors.push(err);
                None
            }
        })
        .collect();
    (entry_list, errors)
}

fn loop_fn(names: &mut [OsString], path: &Path, splits: usize, vec_cap: usize) -> Vec<Result<Elem, FileError>> {
    if splits > 0 && names.len() > 1 {
        let new_split = rayon::current_num_threads().next_power_of_two() + 1;
        let (left, right) = names.split_at_mut(names.len() / 2);
        let (mut lhs, mut rhs) = rayon::join_context(
            |ctx| loop_fn(left, path, if ctx.migrated() { new_split } else {splits-1}, vec_cap),
            |ctx| loop_fn(right, path, if ctx.migrated() { new_split } else { splits-1 }, right.len()));
        lhs.append(&mut rhs);
        lhs
    } else {
        let mut vec = Vec::with_capacity(vec_cap);
        vec.extend(names.iter_mut().map(|name| read_dir_or_file(std::mem::take(name), path, splits).map_err(|err| err.label(path.to_owned()))));
        vec
    }
}

fn read_dir_or_file(name: OsString, path: &Path, splits: usize) -> io::Result<Elem> {
    let path = PathBuf::from(path).join(&name);
    let meta = fs::symlink_metadata(&path)?;
    if meta.is_dir() {
        let read_dir = fs::read_dir(&path)?;
        let mut dir = Dir::new(name);
        let (mut child_names, errors) = read_dir_entry(read_dir);
        let cap = child_names.len();
        let children: Vec<Result<Elem, FileError>> = loop_fn(&mut child_names, &path, splits, cap);
        for child in children {
            match child {
                Ok(Elem::Dir(d)) => dir.dirs.push(d),
                Ok(Elem::File(f)) => dir.files.push(f),
                Ok(Elem::Other) => {}
                Err(err) => dir.errors.push(err),
            }
        }
        Ok(Elem::Dir(dir))
    } else if meta.is_file() || meta.is_symlink() {
        let file = File::new(name, meta.size());
        Ok(Elem::File(file))
    } else {
        // Probably a device file, fifo, etc.
        // doesn't take up space on the disk
        Ok(Elem::Other)
    }
}
