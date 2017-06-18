mod unit;
mod config;
mod parse;
mod exec;

use unit::*;
use config::*;
use std::env;


fn main() {
	let units = match load_units_list()
	{
	Some(new_units) => new_units,
	None => {
		println!("Failed to load units database from file.");
		return;
	},
	};

	let mut args: Vec<String> = Vec::with_capacity(4); // at least 4 args needed. first is prog name

	for arg in env::args()
	{
		args.push(arg);
	}

	if args.len() < 4
	{
		println!("Not enough args. Need #### <input_unit> <output_unit>");
		return;
	}
	
	let input_val = match args[1].parse::<f64>()
	{
	Ok(val) => val,
	Err(err) => {
		println!("Error: {}", err);
		return;
	},
	};
	
	let mut args_iter = args.drain(..);
	args_iter.next(); // skip the prog name
	args_iter.next(); // skip the input value. already got

	println!("{}", exec::convert(input_val, exec::NO_PREFIX, args_iter.next().unwrap(),
			exec::NO_PREFIX, args_iter.next().unwrap(), &units));
}
