use std::{
    ffi::{OsStr, OsString}, fmt, fs, io, os::unix::fs::MetadataExt, path::{Path, PathBuf}
};

mod fs_crossing;
pub mod parallel;
pub mod serial;

pub struct Config {
    pub follow_symlinks: bool,
    pub same_filesystem: bool,
}

impl Config {
    pub fn new(follow_symlinks: bool, same_filesystem: bool) -> Self {
        Config {
            follow_symlinks,
            same_filesystem,
        }
    }
}

struct WalkContext {
    config: Config,
    root_fs: u64,
}


#[derive(Debug)]
pub struct Sizes {
    pub files_size: u64,
    pub total_size: u64,
}

impl Default for Sizes {
    fn default() -> Self {
        Sizes {
            files_size: 0,
            total_size: 0,
        }
    }
}

pub struct CacheOsStr {
    os_str: Option<OsString>,
    string: String,
}

impl fmt::Debug for CacheOsStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        AsRef::<str>::as_ref(self).fmt(f)
    }
}

#[derive(Debug)]
pub struct FileError {
    pub file: PathBuf,
    pub error: io::Error,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.file.display(), self.error)
    }
}

trait LabelError {
    fn take_label(self, file: PathBuf) -> FileError;
    fn label<P: AsRef<Path>>(self, file: P) -> FileError
    where
        Self: Sized,
    {
        self.take_label(file.as_ref().to_owned())
    }
}

impl LabelError for io::Error {
    fn take_label(self, file: PathBuf) -> FileError {
        FileError { file, error: self }
    }
}

impl From<OsString> for CacheOsStr {
    fn from(os_str: OsString) -> Self {
        match os_str.into_string() {
            Ok(string) => CacheOsStr {
                os_str: None,
                string,
            },
            Err(os_string) => {
                let string = os_string.to_string_lossy().into_owned();
                CacheOsStr {
                    os_str: Some(os_string),
                    string,
                }
            }
        }
    }
}

impl AsRef<OsStr> for CacheOsStr {
    fn as_ref(&self) -> &OsStr {
        if let Some(os_str) = &self.os_str {
            os_str.as_ref()
        } else {
            self.string.as_ref()
        }
    }
}

impl AsRef<str> for CacheOsStr {
    fn as_ref(&self) -> &str {
        self.string.as_ref()
    }
}

#[derive(Debug)]
pub struct Dir {
    name: CacheOsStr,
    files: Vec<File>,
    dirs: Vec<Dir>,
    size: Sizes,
}

impl Dir {
    fn new(name: OsString) -> Self {
        Dir {
            name: name.into(),
            files: Vec::new(),
            dirs: Vec::new(),
            size: Sizes::default(),
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn get_dirs(&self) -> &Vec<Dir> {
        &self.dirs
    }

    pub fn get_files(&self) -> &Vec<File> {
        &self.files
    }

    pub fn get_size(&self) -> &Sizes {
        &self.size
    }
}

// TODO: Open and close fd
#[derive(Debug)]
pub struct File {
    name: CacheOsStr,
    size: u64,
}

impl File {
    fn new(name: OsString, size: u64) -> Self {
        File {
            name: name.into(),
            size,
        }
    }

    fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    fn get_size(&self) -> u64 {
        self.size
    }
}

#[derive(Debug)]
enum Elem {
    File(File),
    Dir(Dir),
}

pub fn print_tree(root: &Dir, indent: u32) {
    let indent_str = " ".repeat(indent as usize);
    println!("{}{}", indent_str, root.get_name());
    for file in root.files.iter() {
        println!("{} {}", indent_str, file.get_name());
    }
    for dir in root.dirs.iter() {
        print_tree(dir, indent + 1);
    }
}

fn read_dir_entry<F: FnMut(FileError)>(path: &Path, context: &WalkContext, mut err_collect: F) -> Vec<Elem> {
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
                let meta = entry.metadata().map_err(|err| err.label(path))?;
                if context.config.same_filesystem {
                    if meta.dev() != context.root_fs {
                        return Ok(None);
                    }
                }
                let mut file_type = meta.file_type();
                if file_type.is_symlink() && context.config.follow_symlinks {
                    file_type = fs::metadata(entry.path()).map_err(|err| err.label(path))?.file_type();
                }

                let elem = if file_type.is_dir() {
                    Some(Elem::Dir(Dir::new(entry.file_name())))
                } else if file_type.is_file() || file_type.is_symlink() {
                    Some(Elem::File(File::new(entry.file_name(), meta.len())))
                } else {
                    None
                };
                Ok(elem)
            })
        })
        .filter_map(|res| res.map_err(&mut err_collect).ok().flatten())
        .collect()
}
