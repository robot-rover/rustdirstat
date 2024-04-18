use std::{path::Path, io};

use super::{Dir, OpenDir, OpenElem, Elem};


pub fn parse_tree<P: AsRef<Path>>(path: P) -> (Dir, Vec<io::Error>) {
    let name = path.as_ref().to_string_lossy().to_string();
    let mut dir_stack: Vec<OpenDir> = vec![OpenDir::with_name(name, path).unwrap()];
    let mut err_list = Vec::new();
    loop {
        let mut top = dir_stack.pop().unwrap();
        if let Some(entry) = top.iter.next() {
            let after_top = match entry.and_then(Elem::from_entry) {
                Ok(Some(OpenElem::Dir(dir))) => Some(dir),
                Ok(Some(OpenElem::File(file))) => {
                    top.dir.children.push(Elem::File(file));
                    None
                }
                Ok(None) => None, // ignored file (link, etc)
                Err(err) => {
                    err_list.push(err);
                    None
                }
            };
            dir_stack.push(top);
            if let Some(after_top) = after_top {
                dir_stack.push(after_top);
            }
        } else {
            match dir_stack.last_mut() {
                Some(parent) => parent.dir.children.push(Elem::Dir(top.dir)),
                None => return (top.dir, err_list),
            }
        }
    }
}
