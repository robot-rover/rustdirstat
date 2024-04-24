use std::{env::args, io};

mod parse_tree;
mod gui;

fn main() -> io::Result<()> {
    gui::main().unwrap();
    return Ok(());
    let alg = args().nth(1).unwrap_or_else(|| "serial".to_string());
    let func = match alg.as_str() {
        "serial" => parse_tree::serial::parse_tree,
        "parallel" => parse_tree::parallel::parse_tree,
        _ => panic!("Invalid algorithm: {}", alg),
    };
    let (tree, errors) = func("/")?;
    println!("Error Count: {}, Files Size: {:.3} GB, Total Size: {:.3} GB", errors.len(), tree.get_size().files_size as f64 / 1e9, tree.get_size().total_size as f64 / 1e9);
    Ok(())
}
