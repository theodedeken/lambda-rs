extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod check;
pub mod eval;
pub mod parser;
pub mod sym_tab;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

pub fn read_file(path: &str) -> Result<String, Box<Error>> {
    let mut f = File::open(path)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}
