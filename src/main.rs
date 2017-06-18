mod types;
mod config;
mod database;
mod parse;

use types::*;
use config::*;
use database::*;
use std::io;


fn main() {
	let units = match load_units_list()
	{
	Some(new_units) => new_units,
	None => {
		println!("Failed to load units database from file.");
		return;
	},
	};

	let stdin = io::stdin();
	let mut line_buf = String::with_capacity(80); // std terminal width

	loop
	{
		match stdin.read_line(&mut line_buf)
		{
		Ok(n)    => (),
		Err(err) => println!("Error reading from stdin: {}", err),
		};

		if line_buf.trim() == "exit"
		{
			break;
		}

		if let Some(unit) = units.query(line_buf.trim().to_string())
		{
			println!("Unit was found:\n{:?}", unit);
		}

		line_buf.clear();
	}
}
