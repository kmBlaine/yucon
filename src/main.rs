extern crate regex;
extern crate lazy_static;

mod types;
mod config;
mod database;

use types::*;
use config::*;
use database::*;

fn main() {
    let units_list = load_units_list();
    
    for element in &units_list
    {
        println!( "{}", element );
    }
}
