mod types;
mod config;
mod database;
mod parse;

use types::*;
use config::*;
use database::*;

fn main() {
    load_units_list();
}
