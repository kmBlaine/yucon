mod unit;
mod config;
mod parse;
mod exec;

use unit::*;
use config::*;
use std::env;

static HELP_MSG: &'static str = 
"YUCON - General Purpose Unit Converter - v0.2\n\
 Usage:\n  \
   yucon [options] #.# <input_unit> <output_unit>\n\
   \n  \
   In first form, perform conversion given on the command line\n\
 \n\
 Options:\n  \
   -s         : simple output format. value only\n  \
   -l         : long output format. input / output values and units\n  \
   --help     : show this help message\n  \
   --version  : show version and license info\n\
 \n\
 Examples:\n  \
   $ yucon -l 1 in mm\n    \
     1 in = 25.4 mm\n\
 \n  \
   $ yucon 0 C\n    \
     32.02 F\n
 This is free software licensed under the GNU General Public License v3\n
 Use \'--version\' for more details\n\
 Copyright (C) 2016-2017 Blaine Murphy";

static VERSION_MSG: &'static str =
"YUCON - General Purpose Unit Converter - v0.2\n  \
   Copyright (C) 2016-2017 Blaine Murphy\n  \
   Released 19 Jun 2017 - commit [COMMIT NUM HERE]\n  \
   Source code available at <https://github.com/kmBlaine/yucon>\n  \
   See changelog for version specific details\n  \
   License: GNU Public License v3+\n\
   \n  \
   This program is free software: you can redistribute it and/or modify it under\n  \
   the terms of GPLv3 or any later version. You should have recieved a copy along\n  \
   with this program. If not, see <https://gnu.org/licenses/gpl.html>.\n  \
   \n  \
   There is NO WARRANTY, to the extent permitted by law. See license for more\n  \
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

				return Err("unknown option".to_string());
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

	let input_val = args[0].parse::<f64>().ok().unwrap();

	let mut args_iter = args.drain(..);
	args_iter.next(); // skip the input value. already got

	let mut conversion = exec::convert(input_val, exec::NO_PREFIX, args_iter.next().unwrap(),
			exec::NO_PREFIX, args_iter.next().unwrap(), &units);
	conversion.format = opts.format;

	println!("{}", conversion);
}
