use std::{env::args, io};

use crate::parse_tree::Config;

mod gui;
mod parse_tree;

fn main() -> io::Result<()> {
    gui::main().unwrap();
    return Ok(());
    // let alg = args().nth(1).unwrap_or_else(|| "serial".to_string());
    // let func = match alg.as_str() {
    //     "serial" => parse_tree::serial::parse_tree,
    //     "parallel" => parse_tree::parallel::parse_tree,
    //     _ => panic!("Invalid algorithm: {}", alg),
    // };
    // let config = Config::new(false, true);
    // let (tree, errors) = func("/mnt/data", config)?;
    // println!("Error Count: {}, Files Size: {:.3} GB, Total Size: {:.3} GB", errors.len(), tree.get_size().files_size as f64 / 1e9, tree.get_size().total_size as f64 / 1e9);
    // Ok(())
}
