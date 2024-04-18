
use std::{ffi::{OsStr, OsString}, fs, io};

mod serial;
mod accum_tree;

pub use serial::parse_tree as parse_tree;

pub struct CacheOsStr {
    os_str: Option<OsString>,
    string: String,
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
            },
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

pub struct Dir {
    name: CacheOsStr,
    unparsed_children: Vec<OsString>,
    files: Vec<File>,
    dirs: Vec<Dir>,
    errors: Vec<io::Error>,
}

impl Dir {
    fn new(name: OsString, contents: fs::ReadDir) -> Self {
        let mut err_list = Vec::new();
        let unparsed_children = contents.filter_map(|res| res.map_err(|err| err_list.push(err)).ok()).map(|entry| entry.file_name()).collect();

        Dir {
            name: name.into(),
            unparsed_children,
            files: Vec::new(),
            dirs: Vec::new(),
            errors: err_list,
        }
    }

    fn get_name(&self) -> &str {
        self.name.as_ref()
    }
}

// TODO: Open and close fd
pub struct File {
    name: CacheOsStr,
    size: u64,
}

impl File {
    fn new(name: OsString, size: u64) -> Self {
        File { name: name.into(), size }
    }

    fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    fn get_size(&self) -> u64 {
        self.size
    }
}

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