use super::Dir;

struct Node {
    name: String,
    files_size: u64,
    total_size: u64,
    children: Vec<Node>,
    files: Vec<File>,
}

struct File {
    name: String,
    size: u64,
}

impl Node {
    fn from_dir(dir: Dir) -> Self {
        unimplemented!()
    }
}

fn generate_tree(root: &Dir) -> Node {
    unimplemented!()
}