
use std::{fs, io, path::Path};

mod serial;
mod accum_tree;

pub use serial::parse_tree as parse_tree;
pub enum Elem {
    Dir(Dir),
    File(File),
}

enum OpenElem {
    Dir(OpenDir),
    File(File),
}

impl Elem {
    fn from_entry(entry: fs::DirEntry) -> io::Result<Option<OpenElem>> {
        let ft = entry.file_type()?;
        if ft.is_file() {
            // TODO: Avoid Sockets
            Ok(Some(OpenElem::File(File::new(entry)?)))
        } else if ft.is_dir() {
            Ok(Some(OpenElem::Dir(OpenDir::new(entry.path())?)))
        } else {
            Ok(None)
        }
    }
}

pub struct Dir {
    name: OsStr,
    unparsed_children: Vec<OsStr>,
    files: Vec<File>,
    dirs: Vec<Dir>,
}

impl Dir {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        Dir::with_name(
            path.as_ref()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        )
    }

    fn with_name(name: String) -> Self {
        Dir {
            name,
            children: Vec::new(),
        }
    }

    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn get_children(&self) -> &Vec<Elem> {
        &self.children
    }
}

// TODO: Open and close fd
struct OpenDir {
    dir: Dir,
    iter: fs::ReadDir,
}

impl OpenDir {
    fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(OpenDir {
            dir: Dir::new(&path),
            iter: fs::read_dir(&path)?,
        })
    }

    fn with_name<P: AsRef<Path>>(name: String, path: P) -> io::Result<Self> {
        Ok(OpenDir {
            dir: Dir::with_name(name),
            iter: fs::read_dir(path)?,
        })
    }
}

pub struct File {
    name: String,
    size: u64,
}

impl File {
    fn new(entry: fs::DirEntry) -> io::Result<Self> {
        Ok(File {
            name: entry
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            size: entry.metadata()?.len(),
        })
    }

    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn get_size(&self) -> u64 {
        self.size
    }
}

pub fn print_tree(elem: &Elem, indent: u32) {
    for _i in 0..indent {
        print!(" ");
    }
    match elem {
        Elem::Dir(dir) => {
            println!("+ {}", dir.name);
            for elem in dir.children.iter() {
                print_tree(elem, indent + 1)
            }
        }
        Elem::File(file) => println!(" {}: {}", file.name, file.size),
    }
}