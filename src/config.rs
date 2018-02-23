/* config.rs
 * ===
 * Contains the functions for reading the units.cfg file and compiling it into the units
 * database.
 *
 * This file is part of:
 *
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

use std::error;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use ::parse;
use ::parse::*;
use ::unit;
use ::unit::*;
use std::rc;
use std::rc::Rc;
use std::num::ParseFloatError;
use std::env;


/* enum ParsePropertyError
 *
 * Description: ParsePropertyError is an error sum type for use when parsing
 *   the units.cfg file. The errors it encompasses are:
 *
 *     - SyntaxError: returned when a line violates syntax rules. (see syntax
 *         rules for units.cfg)
 *
 *     - NoSuchProperty: returned when the key in a key-value pair does not
 *         match any known unit properties
 *
 *     - NoSuchType: returned when type requested by the user (eg "length") is
 *         not recognized
 *
 *     - EmptyField: returned when the value in a key value pair is empty.
 *         Technically this a subset of SyntaxError but it is difficult to
 *         determine during tonization and much easier to detect during sematic
 *         analysis.
 *
 *     - InvalidField: returned when the value in a key value pair was not a
 *         (legal) number as expected
 *
 * Usage:
 *   SyntaxError(usize, String):
 *     - usize  : column number at which syntax error occurred.
 *     - String : message describing the violation.
 *
 *   NoSuchProperty(String):
 *     - String : the unrecognized key
 *
 *   NoSuchType(String):
 *     - String : the unrecognized type
 *
 *   EmptyField(String):
 *     - String : the property that which recieved a blank field
 *
 *   InvalidField(std::num::ParseFloatError):
 *     - std::num::ParseFloatError : the underlying error from attempting to
 *                 parse a line as a number
 */
#[derive(Debug)]
enum ParsePropertyError
{
	SyntaxError(SyntaxError),
	NoSuchProperty(String),
	NoSuchType    (String),
	EmptyField    (String),
	InvalidField  (ParseFloatError),
}

impl Error for ParsePropertyError
{
	fn description(&self) -> &str
	{
		match *self
		{
		ParsePropertyError::SyntaxError(ref err)    => err.description(),
		ParsePropertyError::NoSuchProperty(_) => "no such unit property exists",
		ParsePropertyError::NoSuchType(_)     => "no such unit type is recognized by Yucon",
		ParsePropertyError::EmptyField(_)     => "expected value(s) after delimiters \'=\' and \'[\'",
		ParsePropertyError::InvalidField(ref err)   => err.description(),
		}
	}

	fn cause(&self) -> Option<&Error>
	{
		match *self
		{
		ParsePropertyError::InvalidField(ref err) => Some( err ),
		_ => None,
		}
	}
}

impl Display for ParsePropertyError
{
	fn fmt( &self, f: &mut Formatter ) -> fmt::Result
	{
		match *self
		{
		ParsePropertyError::SyntaxError(ref err)       => write!(f, "{}", err),
		ParsePropertyError::InvalidField(ref err )     => write!(f, "bad field value: {}", err.description() ),
		ParsePropertyError::EmptyField(ref prop )      => write!(f, "for property \'{}\': {}", prop, self.description() ),
		ParsePropertyError::NoSuchType(ref unit_type ) => write!(f, "at token \'{}\': {}", unit_type, self.description() ),
		ParsePropertyError::NoSuchProperty(ref prop )  => write!(f, "at token \'{}\': {}", prop, self.description() ),
		}
	}
}

impl From<ParseFloatError> for ParsePropertyError
{
	fn from(err: ParseFloatError) -> ParsePropertyError
	{
		ParsePropertyError::InvalidField( err )
	}
}

impl From<SyntaxError> for ParsePropertyError
{
	fn from(err: SyntaxError) -> ParsePropertyError
	{
		ParsePropertyError::SyntaxError(err)
	}
}
// END ParsePropertyError

/* enum UnitProperty
 *
 * Description: sum type for describing the properties of units. used to make
 *   conveniently representable as computer-friendly object code, allowing
 *   easy analysis and return values for functions. See Yucon docs for more
 *   details on unit properties. Properties encompassed:
 *
 *     - CommonName: the units common name denoted in []
 *
 *     - UnitType: the unit's type eg. length, volume, etc
 *
 *     - ConvFactor: the unit's value in the base unit used for conversion
 *
 *     - Aliases: the unit's aliases and/or abreviations
 *
 *     - ZeroPoint: the unit's zero point if 0 units != 0 base units
 *
 *     - Dimensions: the unit's dimensions. eg meter has 1 but square meter has 2
 *
 *     - Inverse: if the unit is inverse of base unit eg. mpg and L/100km
 */
#[derive(Debug)]
enum UnitProperty
{
	CommonName (String),
	UnitType   (&'static str),
	ConvFactor (f64),
	Aliases    (Vec<Rc<String>>),
	ZeroPoint  (f64),
	Dimensions (u8),
	Inverse    (bool),
}

/* enum PropCheckState
 *
 * Description: Sum type for the states of the UnitPropertyCheck syntax. Used
 *   instead of numeric constants to make each state strictly separate via
 *   Rust's type system and avoid programmer error of overlapping constants.
 *     - OpenBrace  : expecting [
 *     - CloseBrace : expecting ]
 *     - Equals     : expecting =
 *     - Comma      : expecting ,
 *     - Key        : expecting first token of line
 *     - CommonName : expecting token between []
 *     - Value      : expecting token(s) after =
 *     - Validate   : expecting only trailing whitespace or comments
 */
#[derive(Debug)]
enum PropCheckState
{
	OpenBrace,
	CloseBrace,
	Equals,
	Comma,
	Key,
	CommonName,
	Value,
	Validate,
}

/* struct UnitPropertyCheck
 *
 * Description: UnitPropertyCheck is the syntax for the units.cfg file. See
 *   above for exact details on unit declaration syntax.
 *
 * Fields:
 *   - line    : the line currently be analyzed. carried for debug purposes
 *   - esc_set : tracks whether the next character during tokenization will
 *               escaped or not. true if it will be, false otherwise.
 *   - single_val_field : tracks whether a key value pair expects exactly one
 *               value or can recieve a list. true if it expects exactly
 *               one, false otherwise. NOTE: only "aliases" can have a list
 *   - state   : tracks what to expect next. eg [, key, value, etc
 *   - valid   : tracks whether the syntax was vioalted at any point. true if
 *               it is valid syntax, false otherwise.
 */
#[derive(Debug)]
struct UnitPropertyCheck<'a>
{
	line: &'a str,
	esc_set: bool,
	single_val_field: bool,
	state: PropCheckState,
	valid: bool,
}

impl<'a> UnitPropertyCheck<'a>
{
	/* Creates and returns new syntax checker for the given line.
	 *
	 * Parameters:
	 *   - from_line : line of text to be checked
	 */
	fn new(from_line: &'a str) -> UnitPropertyCheck
	{
		UnitPropertyCheck { line:    from_line,
		                    single_val_field: false,
		                    esc_set: false,
		                    state:   PropCheckState::Key,
		                    valid:   true }
	}

	/* Checks if the given delimiter was expected. Returns Ok(true) if it was
	 * or a SyntaxError if it was not. Conceptually just a finite state machine.
	 *
	 * Parameters:
	 *   - token : token to be checked. token is assumed to be a delimiter
	 *   - index : line index where tokenization left off
	 */
	fn check_delim(&mut self, token: &str, index: usize) -> bool
	{
		match self.state
		{
		PropCheckState::OpenBrace => {
			if token == "["
			{
				self.state = PropCheckState::CommonName;
			}
			else
			{
				self.valid = false;
				return false;
			}
		},
		PropCheckState::CloseBrace => {
			if token == "]"
			{
				self.state = PropCheckState::Validate;
			}
			else
			{
				self.valid = false;
				return false;
			}
		},
		PropCheckState::Equals => {
			if token == "="
			{
				self.state = PropCheckState::Value;
			}
			else
			{
				self.valid = false;
				return false;
			}
		},
		PropCheckState::Comma => {
			if token == ","
			{
				self.state = PropCheckState::Value;
			}
			else
			{
				self.valid = false;
				return false;
			}
		},
		PropCheckState::Validate => {
			self.valid = false;
			return false;
		},
		_ => {
			println!("FATAL PARSE ERROR!\n\
			          In line {:?}\n\
			          At index {}\n\
			          Syntax state: {:?}\n\
			          This error should never occur. Please report!",
			          self.line,
			          index,
			          self);
			panic!("syntax check reached impossible state");
		}
		};

		true
	}

	/* Checks if the given token was expected. Returns Ok(true) if it was
	 * or a SyntaxError if it was not. Conceptually just a finite state machine.
	 *
	 * Parameters:
	 *   - token : token to be checked. token is assumed to not be a delimiter
	 *   - index : line index where tokenization left off
	 */
	fn check_normal(&mut self, token: &str, index: usize) -> bool
	{
		match self.state
		{
		PropCheckState::Key => {
			if token.trim().is_empty()
			{
				self.state = PropCheckState::OpenBrace;
			}
			else if token.trim() != "aliases"
			{
				self.state = PropCheckState::Equals;
				self.single_val_field = true;
			}
			else
			{
				self.state = PropCheckState::Equals;
			}
		},
		PropCheckState::Value => {
			if self.single_val_field
			{
				self.state = PropCheckState::Validate;
			}
			else
			{
				self.state = PropCheckState::Comma;
			}
		},
		PropCheckState::CommonName => {
			self.state = PropCheckState::CloseBrace;
		},
		PropCheckState::Validate => {
			if !token.trim().is_empty()
			{
				self.valid = false;
				return false;
			}
		},
		_ => {
			println!("FATAL PARSE ERROR!\n\
			          In line {:?}\n\
			          At index {}\n\
			          Syntax state: {:?}\n\
			          This error should never occur. Please report!",
			          self.line,
			          index,
			          self);
			panic!("syntax check reached impossible state");
		},
		};

		true
	}
}

// See SyntaxChecker trait summary of the methods below
impl<'a> SyntaxChecker for UnitPropertyCheck<'a>
{
	fn feed_token(&mut self, token: &str, delim: bool, index: usize) -> bool
	{
		if delim
		{
			return self.check_delim(token, index);
		}
		else
		{
			self.check_normal(token, index)
		}
	}

	fn assert_valid(&self, index: usize, more_tokens: bool) -> Result<(), SyntaxError>
	{
		// the following states are both invalid exit states and possible error states
		if !more_tokens || !self.valid
		{
			match self.state
			{
			PropCheckState::CloseBrace => {
				return Err(SyntaxError::Expected(index, "\']\'".to_string()));
			},
			PropCheckState::Equals => {
				return Err(SyntaxError::Expected(index, "\'=\'".to_string()));
			},
			PropCheckState::CommonName => {
				return Err(SyntaxError::Expected(index, "token after \'[\'".to_string()));
			},
			_ => (), // all others may not meet criteria. do nothing
			};
		}

		// the following are valid exit states but may still be error states
		if !self.valid
		{
			match self.state
			{
			PropCheckState::OpenBrace => {
				return Err(SyntaxError::Expected(index, "\'[\'".to_string()));
			},
			PropCheckState::Comma => {
				return Err(SyntaxError::Expected(index, "\',\'".to_string()));
			},
			PropCheckState::Validate => {
				return Err(SyntaxError::Expected(index, "whitespace or comment".to_string()));
			},
			_ => (), // Key and Value states are always okay to exit on. Just do nothing
			};
		}

		Ok(())
	}

	fn is_esc(&self, ch: char) -> bool
	{
		ch == '\\'
	}

	fn is_comment(&self, ch: char) -> bool
	{
		ch == '#'
	}

	fn is_delim(&self, ch: char) -> bool
	{
		ch == '[' ||
		ch == ']' ||
		ch == ',' ||
		ch == '='
	}

	fn is_preserved_delim(&self, ch: char) -> bool
	{
		false
	}

	fn esc_char(&self) -> char
	{
		'\\'
	}

	fn valid(&self) -> bool
	{
		self.valid
	}

	fn esc_set(&self) -> bool
	{
		self.esc_set
	}

	fn set_esc(&mut self, set: bool)
	{
		self.esc_set = set;
	}

	fn reset(&mut self)
	{
		self.valid = true;
		self.state = PropCheckState::Key;
		self.esc_set = false;
	}
}

/* Returns the reference to the matching statically allocated unit type string
 * or returns error if that type is not recognized. Helper function for
 * fn parse_key_value to separate out the search code and avoid code blob.
 *
 * Paramemters:
 *   - requested_type : type denoted in a "type =" unit property
 */
fn get_unit_type(requested_type: String) -> Result<&'static str, ParsePropertyError>
{
	{ // scope to avoid borrow problem when handing string to NoSuchType error
	let user_type = requested_type.as_str();

	for unit_type in unit::UNIT_TYPES.iter()
	{
		if *unit_type == user_type
		{
			return Ok(*unit_type);
		}
	}
	} // end borrow scope

	Err(ParsePropertyError::NoSuchType(requested_type))
}

/* Parses value part of a key-value pair as a number. Helper function for
 * fn parse_key_value to avoid duplicate code. Returns (bool, f64) tuple
 * if the field was empty or a valid number, InvalidField error otherwise.
 * The bool part of the return is false if the field was not empty / valid
 * and true if it was empty (Option::None). Helper function for
 * fn parse_key_value to avoid duplicated code for numeric key-value pairs.
 *
 * Parameters:
 *   - field : token retrieved directly from an iterator and thus is Option
 */
fn field_as_num(field: Option<TokenType>) -> Result<(bool, f64), ParsePropertyError>
{
	let token = match field
	{
		None      => return Ok((true, ::std::f64::NAN)),
		Some(val) => val,
	};

	let value = try!(token.unwrap().parse::<f64>());

	Ok((false, value))
}

/* Parses a key-value unit property. Returns the associated unit property if it
 * is a valid pair or error if:
 *   - the key is not a recognized property
 *   - the value is missing
 *   - type mismatch for the value
 * Helper function for fn parse_line to separate out the semantic analysis and
 * avoid a code glut.
 *
 * Parameters:
 *   - tokens : vector of tokens to be parsed. blank tokens expected to be
 *              filtered and syntax expected to have already been validated
 */
fn parse_key_value(mut tokens: Vec<TokenType>) -> Result<UnitProperty, ParsePropertyError>
{
	let mut tokens_iter = tokens.drain(..);
	let mut field_empty = true;
	let key = tokens_iter.next().unwrap().unwrap();

	let unit_property = match key.as_str()
	{
	"aliases" => {
		let mut aliases = Vec::new();

		for token in tokens_iter
		{
			match token
			{
			TokenType::Normal(tok) => {
				aliases.push(Rc::new(tok));
				field_empty = false;
			}
			_ => (),
			};
		}

		UnitProperty::Aliases(aliases)
	},
	"conv_factor" => {
		tokens_iter.next();
		let (empty, conv_factor) = try!(field_as_num(tokens_iter.next()));
		field_empty = empty;
		UnitProperty::ConvFactor(conv_factor)
	},
	"dimensions" => {
		tokens_iter.next();
		let (empty, reqested_dims) = try!(field_as_num(tokens_iter.next()));
		field_empty = empty;
		let dims: u8 = if reqested_dims <= u8::max_value() as f64
		{
			reqested_dims as u8
		}
		else
		{
			// @TODO Change this a formal error as the default is already 1.
			println!("\n*** WARNING ***\n\
			          Requested {} dimensions for a unit. \
			          Yucon allows at most 255. Using default (1).",
			          reqested_dims);
			1
		};
		UnitProperty::Dimensions(dims)
	},
	"inverse" => {
		tokens_iter.next();
		let (empty, value) = try!(field_as_num(tokens_iter.next()));
		field_empty = empty;
		let inverse = if value == 0.0
		{
			false
		}
		else
		{
			true
		};
		UnitProperty::Inverse(inverse)
	}
	"type" => {
		tokens_iter.next();

		let unit_type = match tokens_iter.next()
		{
			None      => unit::UNIT_TYPES[0], // technically an error but this will be caught later by the empty field check
			Some(val) => {
				field_empty = false;
				try!(get_unit_type(val.unwrap()))
			},
		};

		UnitProperty::UnitType(unit_type)
	},
	"zero_point" => {
		tokens_iter.next();
		let (empty, zero_point) = try!(field_as_num(tokens_iter.next()));
		field_empty = empty;
		UnitProperty::ZeroPoint(zero_point)
	},
	_ => return Err(ParsePropertyError::NoSuchProperty(key)),
	};

	if field_empty
	{
		return Err(ParsePropertyError::EmptyField(key));
	}

	Ok(unit_property)
}

/* Parses the common name unit property. Returns the common name if it exists.
 * wrapped in a UnitProperty. Returns EmptyField if the common name is not given
 * Helper function for fn parse_line to separate out semantic analysis and avoid
 * a code glut.
 *
 * Parameters:
 *   - tokens : vector of TokenType wrapped tokens. this vector is expected to
 *              have empty tokens filtered out and to have common name syntax
 *              already validated
 */
fn parse_common_name(mut tokens: Vec<TokenType>) -> Result<UnitProperty, ParsePropertyError>
{
	// after filtering, the common name field should have exactly 3 tokens
	// '[', 'name', ']' less or more and we have a problem
	if tokens.len() != 3
	{
		return Err(ParsePropertyError::EmptyField("common name".to_string()));
	}

	let mut tokens_iter = tokens.drain(..);
	tokens_iter.next();
	let common_name = tokens_iter.next().unwrap().unwrap();

	Ok(UnitProperty::CommonName(common_name))
}

/* Parses a line from the units.cfg file and returns the unit property described
 * if any as program internal object code. Seeing as lines may be purely
 * whitespace or comments, Option<UnitProperty> is returned instead of
 * UnitProperty directly. Returns an appropriate error for malformed lines that
 * violate syntax or sematics.
 *
 * Valid unit properties are as follows:
 *   - Common Name        : "[name]"
 *   - Aliases            : "aliases = alt1, alt2, alt3"
 *   - Type               : "type = <unit type>"
 *   - Converseion Factor : "conv_factor = 1.2345"
 *   - Dimensions         : "dimensions = 3"
 *   - Inverse            : "inverse = 1"
 *   - Zero Point         : "zero_point = 1.234e5"
 *
 * These are only basic examples. See "doc/UnitsCFG_Explained.md" for
 * full units.cfg syntax and semantics specification .
 *
 * Parameters:
 *   - line : line of input to parse
 */
fn parse_line(line: &str) -> Result<Option<UnitProperty>, ParsePropertyError>
{
	let mut syntax_check = UnitPropertyCheck::new(line);
	let mut raw_tokens = try!(tokenize(line, &mut syntax_check));
	let mut tokens: Vec<TokenType> = Vec::with_capacity(raw_tokens.len());

	for raw_tok in raw_tokens.drain(..)
	{
		let new_tok = match raw_tok
		{
		TokenType::Delim(tok) => {
			TokenType::Delim(tok.trim().to_string())
		},
		TokenType::Normal(tok) => {
			TokenType::Normal(tok.trim().to_string())
		},
		};

		if new_tok.is_empty()
		{
			continue;
		}

		tokens.push(new_tok);
	}

	// if line was whitespace or comment
	// fn tokenize ensures at least one empty token for blank or comment lines
	if tokens.len() == 0 // tokens.len() == 1 && tokens[0].is_empty()
	{
		return Ok(None);
	}

	// tokens.retain(|tok| !tok.is_empty());
	let mut common_name = true;

	match tokens[0]
	{
	TokenType::Delim(ref tok) => {
		if tok != "["
		{
			println!("FATAL PARSE ERROR!\n\
			          In line {:?},\n\
			          tokenized as {:?}\n\
			          This error should never occur. Please report!",
			          line,
			          tokens);
			panic!("illegal delimiter begins line after syntax verification");
		}
	},
	TokenType::Normal(_) => common_name = false,
	};

	let unit_property = if common_name
	{
		try!(parse_common_name(tokens))
	}
	else
	{
		try!(parse_key_value(tokens))
	};

	Ok(Some(unit_property))
}

fn add_unit(database: &mut UnitDatabase, new_unit: Unit, aliases: &Vec<Rc<String>>)
{
	if new_unit.is_well_formed()
	{
		if let Some(unit) = database.add(new_unit, aliases)
		{
			println!("\n*** ERROR ***\n\
			          Failed to add unit {}: an existing unit shares names with this one\n",
			          unit.common_name);
		}
	}
	else
	{
		println!("\n*** ERROR ***\n\
		          Failed to add unit {}: unit is missing mandatory properties.\n",
		          new_unit.common_name);
	}
}
fn find_and_make_cfg() -> io::Result<File>
{
	let (default_path, path_sepr) = if cfg!(target_os="linux")
	{
		("/etc/yucon/units.cfg", "/")
	}
	else
	{
		(r"C:\Program Files\Yucon\units.cfg", r"\")
	};

	let mut home_path = match env::home_dir()
	{
		Some(path) => {
			path
		},
		None => {
			return File::open(default_path);
		},
	};

	let mut home_cfg = home_path.clone();
	home_cfg.push(".yucon");
	home_cfg.push("units.cfg");

	match File::open(&home_cfg)
	{
		Err(err) => {
			let default_file = try!(File::open(&default_path));
			home_path.push(".yucon");
			try!(fs::create_dir(home_path));
			try!(fs::copy(&default_path, &home_cfg));
			Ok(default_file)
		},
		Ok(file) => {
			Ok(file)
		},
	}
}

pub fn load_units_list() -> Option<UnitDatabase>
{
	let file = match find_and_make_cfg()
	{
		Err(err) => {
			println!("Unable to open units.cfg: {}", err.description());
			return None;
		},
		Ok(file)  => file,
	};

	let mut units_cfg = BufReader::new(file);
	let mut line = String::with_capacity(80); // standard terminal width. all lines in stock units.cfg will fit in this.
	let mut line_num = -1;
	let mut first_unit = true;

	let mut units_database = UnitDatabase::new();
	let mut new_unit = Unit::new();
	let mut aliases: Vec<Rc<String>> = Vec::new();


	while units_cfg.read_line(&mut line).unwrap() > 0
	{
		line_num += 1;

		match parse_line(&line)
		{
		Ok(wrapper) => {
			if let Some(prop) = wrapper
			{
				// TODO: excessive code. change this block to an inline function for clarity
				match prop
				{
				UnitProperty::CommonName(name) => {
					if first_unit
					{
						new_unit.set_common_name(name);
						first_unit = false;
					}
					else
					{
						add_unit(&mut units_database, new_unit, &aliases);
						new_unit = Unit::new();
						new_unit.set_common_name(name);
					}
				},
				UnitProperty::Aliases(other_names) => {
					if new_unit.has_aliases
					{
						println!("\n*** WARNING ***\n\
						          For unit {}: attempted to assign aliases twice. Ignoring this attempt.\n",
						          new_unit.common_name);
					}
					else
					{
						new_unit.has_aliases = true;
						aliases = other_names;
					}
				},
				UnitProperty::UnitType(unit_type)     => new_unit.set_unit_type(unit_type),
				UnitProperty::ConvFactor(conv_factor) => new_unit.set_conv_factor(conv_factor),
				UnitProperty::ZeroPoint(zero_point)   => new_unit.set_zero_point(zero_point),
				UnitProperty::Dimensions(dimensions)  => new_unit.set_dimensions(dimensions),
				UnitProperty::Inverse(inverse)        => new_unit.set_inverse(inverse),
				};
			};
		},
		Err(err) => {
			println!("\n*** ERROR ***\n\
				      In line {}: \"{}\": \
				      {}\n", line_num, line.trim_right(), err );
		},
		};

		line.clear();
	}

	// units added when a new section begins
	// last unit in file will not be added without this
	add_unit(&mut units_database, new_unit, &aliases);

	Some(units_database)
}
