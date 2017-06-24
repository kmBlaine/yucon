mod unit;
mod config;
mod parse;
mod exec;

use unit::*;
use config::*;
use std::env;

static HELP_MSG: &'static str = "\
YUCON - General Purpose Unit Converter - v0.2
Usage:
  yucon [options] #.# <input_unit> <output_unit>

  In first form, perform conversion given on the command line

Options:
  -s         : simple output format. value only
  -l         : long output format. input / output values and units
  --help     : show this help message
  --version  : show version and license info

Examples:
  $ yucon -l 1 in mm
    1 in = 25.4 mm

  $ yucon 0 C
    32.02 F

This is free software licensed under the GNU General Public License v3
Use \'--version\' for more details
Copyright (C) 2016-2017 Blaine Murphy";

static VERSION_MSG: &'static str = "\
YUCON - General Purpose Unit Converter - v0.2
  Copyright (C) 2016-2017 Blaine Murphy
  Released 24 Jun 2017 - commit [COMMIT NUM HERE]
  Source code available at <https://github.com/kmBlaine/yucon>
  See changelog for version specific details
  License: GNU Public License v3+

This program is free software: you can redistribute it and/or modify it under
the terms of GPLv3 or any later version. You should have recieved a copy along
with this program. If not, see <https://gnu.org/licenses/gpl.html>.

There is NO WARRANTY, to the extent permitted by law. See license for more
details.";

struct Options
{
	batch: bool,
	format: exec::ConversionFmt,
}

impl Options
{
	fn new() -> Options
	{
		Options {
			batch: false,
			format: exec::ConversionFmt::Desc,
		}
	}

	fn get_opts() -> Result<(Options, Vec<String>), String>
	{
		let mut opts = Options::new();
		let mut extras = Vec::with_capacity(env::args().count());
		let mut args = env::args();
		args.next(); // skip program name

		loop
		{
			let arg = match args.next()
			{
			Some(opt) => opt,
			None => break,
			};

			if arg.starts_with("--")
			{
				match arg.as_ref()
				{
				"--help" => return Err(HELP_MSG.to_string()),
				"--version" => return Err(VERSION_MSG.to_string()),
				_ => return Err("unknown option".to_string()),
				};
			}
			else if arg.starts_with("-")
			{
				if arg.parse::<f64>().is_ok()
				{
					extras.push(arg);

					for extra in args
					{
						extras.push(extra);
					}

					if extras.len() < 3
					{
						return Err("not enough args".to_string());
					}

					return Ok((opts, extras));
				}
				else
				{
					let mut chars = arg.chars();
					chars.next(); // get rid of dash
					for ch in chars
					{
						match ch
						{
						's' => opts.format = exec::ConversionFmt::Short,
						'l' => opts.format = exec::ConversionFmt::Long,
						_ => return Err("unknown option".to_string()),
						};
					}
				}
			}
			else
			{
				extras.push(arg);

				for extra in args
				{
					extras.push(extra);
				}

				if extras.len() < 3
				{
					return Err("not enough args".to_string());
				}

				return Ok((opts, extras));
			}
		}

		if !opts.batch && extras.len() < 3
		{
			return Err("not enough args".to_string());
		}

		Ok((opts, extras))
	}
}

fn main() {
	let units = match load_units_list()
	{
	Some(new_units) => new_units,
	None => {
		println!("Failed to load units database from file.");
		return;
	},
	};

	let (opts, mut args) = match Options::get_opts()
	{
	Ok(results) => results,
	Err(msg) => {
		println!("{}", msg);
		return;
	},
	};

	let input_val = match exec::parse_number_expr(&args[0])
	{
	Ok(expr) => {
		println!("Recall is {}", expr.recall);
		expr.value
	},
	Err(err) => {
		println!("In token \'{}\': {}", &args[0], err);
		return;
	},
	};

	let mut args_iter = args.drain(..);
	args_iter.next(); // skip the input value. already got

	let mut conversion = exec::convert(input_val, exec::NO_PREFIX, args_iter.next().unwrap(),
			exec::NO_PREFIX, args_iter.next().unwrap(), &units);
	conversion.format = opts.format;

	println!("{}", conversion);
}
