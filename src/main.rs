/*
 * Yucon - General Purpose Unit Converter
 * Copyright (C) 2016-2017  Blaine Murphy
 *
 * This program is free software: you can redistribute it and/or modify it under the terms
 * of the GNU General Public License as published by the Free Software Foundation, either
 * version 3 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
 * without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
use std::fmt::Write;

static PROGRAM_NAME: &'static str = "\
YUCON - General Purpose Unit Converter - v0.3";

static COPYRIGHT_MSG: &'static str = "\
Copyright (C) 2016-2018 Blaine Murphy";

static HELP_MSG: &'static str = "\
Usage:
  yucon [options]
  yucon [options] <#> <input_unit> <output_unit>

  In first form, run an interactive session for converting units
  In second form, perform conversion given on the command line

Options:
  -s         : simple output format. value only
  -l         : long output format. input / output values and units
  --help     : show this help message
  --version  : show version and license info

Examples:
  Conversion on invocation:
    $ yucon 1 in mm
      25.4 mm

  Interactive session with long formatting:
    $ yucon -l

This is free software licensed under the GNU General Public License v3
Use \'--version\' for more details";

static VERSION_MSG: &'static str = "\
\0  Released 01 Sep 2018
  Source code available at <https://github.com/kmBlaine/yucon>
  See doc/Changelog.md for version specific details
  License: GNU Public License v3+

This program is free software: you can redistribute it and/or modify it under
the terms of GPLv3 or any later version. You should have recieved a copy along
with this program. If not, see <https://gnu.org/licenses/gpl.html>.

There is NO WARRANTY, to the extent permitted by law. See license for more
details.
";

static INTERACTIVE_HELP_MSG: &'static str = "\
Enter a conversion or a command...
Conversions:
  Format: <#> <input_unit> <output_unit>

  #               - value to convert. may be any valid floating point value
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
  output_unit     - recall for unit being converted to";

static GREETING_MSG: &'static str = "\
====
This is free software licensed under the GNU General Public License v3
Type \'version\' for more details";

struct Options
{
    interactive: bool,
    format: exec::ConversionFmt,
}

impl Options
{
    fn new() -> Options
    {
        Options {
            interactive: true,
            format: exec::ConversionFmt::Desc,
        }
    }

    fn get_opts() -> Result<(Options, Vec<String>), InterpretErr>
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
                "--help" => return Err(InterpretErr::HelpSig),
                "--version" => return Err(InterpretErr::VersionSig),
                _ => return Err(InterpretErr::UnknownLongOpt(arg)),
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
                        return Err(InterpretErr::IncompleteErr);
                    }

                    opts.interactive = false;
                    break;
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
                        _ => return Err(InterpretErr::UnknownShortOpt(ch)),
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
                    return Err(InterpretErr::IncompleteErr);
                }

                opts.interactive = false;
                break;
            }
        }

        Ok((opts, extras))
    }
}

fn line_interpreter(units: &UnitDatabase, opts: &Options)
{
    let prompt = "> ".to_string();
    let mut interpreter: Interpreter<_, _> =
        interpret::Interpreter::using_streams(stdin(), stdout());

    interpreter.format = opts.format;
    interpreter.publish(&PROGRAM_NAME, &None);
    interpreter.newline();
    interpreter.publish(&GREETING_MSG, &None);
    interpreter.newline();
    interpreter.publish(&COPYRIGHT_MSG, &None);
    interpreter.newline();
    interpreter.newline();
    interpreter.publish(&"Enter a conversion or a command. Type \'help\' for assistance.", &None);
    interpreter.newline();

    loop
    {
        interpreter.newline();
        interpreter.publish(&prompt, &None);
        let cmd_result = interpreter.interpret();
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
                    interpreter.publish(&INTERACTIVE_HELP_MSG, &None);
                    interpreter.newline();
                },
                InterpretErr::VersionSig => {
                    interpreter.publish(&PROGRAM_NAME, &None);
                    interpreter.newline();
                    interpreter.publish(&VERSION_MSG, &None);
                    interpreter.newline();
                    interpreter.publish(&COPYRIGHT_MSG, &None);
                    interpreter.newline();
                },
                InterpretErr::CmdSuccess(..) => {
                    interpreter.publish(&cmd_mesg, &None);
                    interpreter.newline();
                }
                _ => {
                    interpreter.publish(&cmd_mesg, &Some("Error: ".to_string()));
                    interpreter.newline();
                },
                };

                continue;
            },
            Ok(toks) => toks,
        };

        let mut conv_primitive = match exec::to_conv_primitive(&tokens)
        {
            Ok(prim) => prim,
            Err(err) => {
                let mut mesg = String::with_capacity(80);
                write!(mesg, "In token \'{}\': ", tokens[err.failed_at].peek());
                interpreter.publish(&err, &Some(mesg));
                interpreter.newline();
                continue;
            },
        };

        match interpreter.perform_recall(&mut conv_primitive)
        {
        None => {},
        Some(err) => {
            interpreter.publish(&err, &Some("Error: ".to_string()));
            interpreter.newline();
            continue;
        },
        };

        let mut conversions = exec::convert_all(conv_primitive, units);

        for mut conversion in &mut conversions
        {
            conversion.format = interpreter.format;
            interpreter.publish(&conversion, &None);
            interpreter.newline();
        }

        interpreter.update_recall(&conversions);
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
        Err(err) => {
            match err
            {
            InterpretErr::HelpSig => {
                println!("{}", &PROGRAM_NAME);
                println!("{}", &HELP_MSG);
                println!("{}", &COPYRIGHT_MSG);
            },
            InterpretErr::VersionSig => {
                println!("{}", &PROGRAM_NAME);
                println!("{}", &VERSION_MSG);
                println!("{}", &COPYRIGHT_MSG);
            },
            _ => {
                println!("Error: {}", err);
                println!("Use \'--help \' for assistance");
            },
            }
            return;
        },
    };

    if opts.interactive
    {
        line_interpreter(&units, &opts);
    }
    else
    {
        let mut interpreter: Interpreter<_, _> =
                interpret::Interpreter::using_streams(stdin(), stdout());

        interpreter.format = opts.format;
        let mut args_wrapped: Vec<parse::TokenType> = Vec::with_capacity(3);

        for arg in args.drain(..)
        {
            args_wrapped.push(parse::TokenType::Normal(arg));
        }

        let mut conv_primitive = match exec::to_conv_primitive(&args_wrapped)
        {
            Ok(results) => results,
            Err(err) => {
                println!("In token \'{}\': {}", args_wrapped[err.failed_at].peek(), err);
                return;
            },
        };

        match interpreter.perform_recall(&mut conv_primitive)
        {
        None => {},
        Some(err) => {
            println!("Error: {}", err);
            return;
        },
        };

        let mut conversions = exec::convert_all(conv_primitive, &units);

        for mut conversion in &mut conversions
        {
            conversion.format = interpreter.format;
            println!("{}", conversion);
        }
    }
}
