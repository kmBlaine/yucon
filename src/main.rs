mod unit;
mod config;
mod parse;
mod exec;
mod interpret;

use unit::*;
use config::*;
use interpret::*;
use std::env;
use std::io::stdin;
use std::io::stdout;

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
  Released 15 Nov 2017
  Source code available at <https://github.com/kmBlaine/yucon>
  See changelog for version specific details
  License: GNU Public License v3+

This program is free software: you can redistribute it and/or modify it under
the terms of GPLv3 or any later version. You should have recieved a copy along
with this program. If not, see <https://gnu.org/licenses/gpl.html>.

There is NO WARRANTY, to the extent permitted by law. See license for more
details.";

static INTERACTIVE_HELP_MSG: &'static str = "\
Enter a conversion or a command...
Conversions:
  Format: <input_val> <input_unit> <output_unit>
  
  input_val       - value to convert. may be any valid floating point value
  input_unit      - unit being converted from
  output_unit     - unit being converted to

Commands:
  exit            - exit the program
  help            - print this help message
  version         - print version and license info
  <var> [<state>] - view or set program variables. view if no state is specified
                    set the variable to given state otherwise

Program Variables:
  format          - output format. may be \'s\', \'d\', or \'l\'
  value           - recall value for conversions
  input_unit      - recall for unit being converted from
  output_unit     - recall for unit being converted to

";

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
			batch: true,
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

fn line_interpreter()
{
	let mut interpreter: Interpreter<_, _> =
		interpret::Interpreter::using_streams(stdin(), stdout());
	
	loop
	{
		let mut cmd_result = interpreter.interpret();
		let tokens = match cmd_result
		{
			Err(cmd_mesg) => {
				match cmd_mesg
				{
				InterpretErr::BlankLine => {
					continue;
				},
				InterpretErr::ExitSig => {
					break;
				},
				InterpretErr::HelpSig => {
					interpreter.publish(&INTERACTIVE_HELP_MSG.to_string(), &None);
				},
				InterpretErr::VersionSig => {
					interpreter.publish(&VERSION_MSG.to_string(), &None);
				},
				InterpretErr::CmdSuccess(..) => {
					interpreter.publish(&cmd_mesg, &None);
				}
				_ => {
					interpreter.publish(&cmd_mesg, &Some("Error: ".to_string()));
				},
				};
				
				continue;
			},
			Ok(toks) => toks,
		};
		
		for token in tokens
		{
			println!("{}", token.peek());
		}
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
	
	if args.is_empty()
	{
		line_interpreter();
	}
	else
	{
		let mut args_wrapped: Vec<parse::TokenType> = Vec::with_capacity(3);
		
		for arg in &args
		{
			args_wrapped.push(parse::TokenType::Normal(arg.clone()));
		}
		
		let mut conv_primitive = match exec::to_conv_primitive(args_wrapped)
		{
			Ok(results) => results,
			Err(err) => {
				println!("In token \'{}\': {}", args[err.failed_at], err);
				return;
			},
		};
	
		println!("Value recall is {}", conv_primitive.input_val.recall);
		println!("Input unit recall is {}", conv_primitive.input_unit.recall);
		println!("Output unit recall is {}", conv_primitive.input_unit.recall);
	
		if conv_primitive.input_unit.alias.is_none()
		{
			conv_primitive.input_unit.alias = Some("m".to_string());
		}
		
		if conv_primitive.output_unit.alias.is_none()
		{
			conv_primitive.output_unit.alias = Some("m".to_string());
		}
	
		let mut conversion = exec::convert_all(conv_primitive, &units);
		
		conversion[0].format = opts.format;
		println!("{}", conversion[0]);
	}
}
