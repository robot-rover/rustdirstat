use std::io;

mod parse_tree;

use parse_tree::parse_tree;

fn main() -> io::Result<()> {
    let (tree, errors) = parse_tree("/home/")?;
    for error in errors {
        eprintln!("{}", error);
    }
    // print_tree(&tree, 0);
    Ok(())
}
