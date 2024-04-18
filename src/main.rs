use std::io;

mod parse_tree;

use parse_tree::{parse_tree, print_tree, Elem};

fn main() -> io::Result<()> {
    let (tree, _errors) = parse_tree("/home/robot_rover/Downloads");
    print_tree(&Elem::Dir(tree), 0);
    Ok(())
}
