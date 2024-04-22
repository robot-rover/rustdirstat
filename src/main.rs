use std::{env::args, io};

use parse_tree::serial::parse_tree;

mod parse_tree;

fn main() -> io::Result<()> {
    let alg = args().nth(1).unwrap_or_else(|| "serial".to_string());
    let func = match alg.as_str() {
        "serial" => parse_tree::serial::parse_tree,
        "parallel" => parse_tree::parallel::parse_tree,
        "parallel2" => parse_tree::parallel2::parse_tree,
        _ => panic!("Invalid algorithm: {}", alg),
    };
    let (tree, errors) = func("/")?;
    for error in errors {
        eprintln!("{}", error);
    }
    // print_tree(&tree, 0);
    Ok(())
}
