use std::io;

mod parse_tree;

use parse_tree::{parse_tree, print_tree};

fn main() -> io::Result<()> {
    let tree = parse_tree("/home/robot_rover/Downloads")?;
    print_tree(&tree, 0);
    Ok(())
}
