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

    // Todo: use threadsafe queue
    if let Elem::Dir(dir) = read_dir_or_file(name, &root_can)? {
        Ok((dir, Vec::new()))
    } else {
        panic!("Root is not a directory: {:?}", &root_can)
    }
}

fn read_dir_entry(contents: fs::ReadDir) -> Vec<io::Result<OsString>> {
    let entry_list = contents
        .map(|res| res.map(|entry| entry.file_name()))
        .collect();
    entry_list
}

pub fn read_dir_or_file(name: OsString, path: &Path) -> io::Result<Elem> {
    let path = PathBuf::from(path).join(&name);
    let meta = fs::symlink_metadata(&path)?;
    if meta.is_dir() {
        let read_dir = fs::read_dir(&path)?;
        let mut dir = Dir::new(name);
        let children = read_dir_entry(read_dir);
        let children: Vec<Result<Elem, FileError>> = children.into_par_iter().map(|res| {
            match res {
                Ok(child_name) => read_dir_or_file(child_name, &path).map_err(|err| err.label(path.to_owned())),
                Err(err) => Err(err.label(path.to_owned())),
            }
        }).collect();
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
