use ::exec::*;
use ::unit::UnitDatabase;
use std::io;
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Stdout;
use std::io::Stdin;
use std::io::stdout;
use std::io::stdin;
use ::parse::*;
use std::fmt;
use std::fmt::Display;
use std::fmt::Write;
use std::error::Error;

#[derive(Debug)]
pub enum InterpretErr
{
	CmdSuccess(String),
	UnrecognizedCmd(String),
	InvalidState(String),
	TokenizeErr(SyntaxError),
	ExitSig,
	BlankLine,
	HelpSig,
	VersionSig,
	ConversionSig
}

impl Error for InterpretErr
{
	fn description(&self) -> &str
	{
		match *self
		{
		InterpretErr::CmdSuccess(..) => "command completed successfully",
		InterpretErr::UnrecognizedCmd(..) => "unrecognized command or variable",
		InterpretErr::InvalidState(..) => "invalid variable state",
		InterpretErr::TokenizeErr(ref err) => err.description(),
		InterpretErr::ExitSig => "user terminated session",
		InterpretErr::BlankLine => "no action",
		InterpretErr::HelpSig => "user requested help",
		InterpretErr::VersionSig => "user requested version",
		InterpretErr::ConversionSig => "user issued conversion",
		}
	}
	
	fn cause(&self) -> Option<&Error>
	{
		match *self
		{
		InterpretErr::TokenizeErr(ref err) => Some(err),
		_ => None,
		}
	}
}

impl Display for InterpretErr
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		match *self
		{
		InterpretErr::CmdSuccess(ref mesg) => {
			write!(f, "{}", mesg)
		},
		InterpretErr::UnrecognizedCmd(ref cmd) => {
			write!(f, "{}: {}", self.description(), cmd)
		},
		InterpretErr::InvalidState(ref state) => {
			write!(f, "{}: {}", self.description(), state)
		},
		InterpretErr::TokenizeErr(ref err) => {
			write!(f, "{}", err)
		}
		_ => {
			write!(f, "{}", self.description())
		},
		}
	}
}

impl From<SyntaxError> for InterpretErr
{
	fn from(err: SyntaxError) -> InterpretErr
	{
		InterpretErr::TokenizeErr(err)
	}
}

struct LineCheck
{
	esc: bool,
	valid: bool,
	argc: u32,
}

impl LineCheck
{
	fn new() -> LineCheck
	{
		LineCheck { esc: false,
		            valid: false,
		            argc: 0,
		}
	}
}

impl SyntaxChecker for LineCheck
{
	fn feed_token(&mut self, token: &str, delim: bool, index: usize) -> bool
	{
		if !delim && !token.is_empty()
		{
			self.argc += 1;
		}
		true
	}
	fn is_esc(&self, ch: char) -> bool
	{
		ch == self.esc_char() && !self.esc
	}
	fn is_comment(&self, ch: char) -> bool
	{
		ch == '#' || ch == '\n'
	}
	fn is_delim(&self, ch: char) -> bool
	{
		ch == ' '
	}
	fn is_preserved_delim(&self, ch: char) -> bool
	{
		ch == ':' ||
		ch == ';' ||
		ch == '_' ||
		(ch == '\\' && self.esc) // need to preserve escape sequences for later parsing
	}
	fn esc_char(&self) -> char
	{
		'\\'
	}
	fn valid(&self) -> bool
	{
		self.valid
	}
	fn assert_valid(&self, index: usize, more_tokens: bool) -> Result<(), SyntaxError>
	{
		Ok(())
	}
	fn esc_set(&self) -> bool
	{
		self.esc
	}
	fn set_esc(&mut self, set: bool)
	{
		self.esc = set;
	}
	fn reset(&mut self)
	{
		self.esc = false;
	}
}

pub struct Interpreter<I, O> where I: Read, O:io:: Write
{
	pub format: ConversionFmt,
	input_stream: BufReader<I>,
	output_stream: O,
	input_unit: Option<String>,
	output_unit: Option<String>,
}

impl <I, O> Interpreter<I, O> where I: Read, O: io::Write
{
	pub fn new() -> Interpreter<Stdin, Stdout>
	{
		Interpreter { format: ConversionFmt::Desc,
		              input_stream: BufReader::new(stdin()),
		              output_stream: stdout(),
		              input_unit: None,
		              output_unit: None,
		}
	}
	
	pub fn using_streams(istream: I, ostream: O) -> Interpreter<I, O>
	{
		Interpreter { format: ConversionFmt::Desc,
		              input_stream: BufReader::new(istream),
		              output_stream: ostream,
		              input_unit: None,
		              output_unit: None,
		}
	}
	
	/* Gets the next line from the input stream and interpets as either a
	 * conversion or a command. If it is a command ie beginning in a program
	 * internal keyword then the command will attempt to be executed and a 
	 * relevant message or error will be returned. If it was not a command and
	 * is of sufficient length to be a conversion, the line will be returned
	 * tokenized and stored as a Vec<TokenType> for further processing. 
	 * 
	 * Returns:
	 *   
	 */
	pub fn interpret(&mut self) -> Result<Vec<TokenType>, InterpretErr>
	{
		let mut raw_line = String::with_capacity(80); // std terminal width
		let bytes_read = self.input_stream.read_line(&mut raw_line);
		
		if bytes_read.is_err()
		{
			write!(self.output_stream, "fatal input stream error: {}", bytes_read.err().unwrap());
			return Err(InterpretErr::ExitSig);
		}
		else if bytes_read.unwrap() == 0
		{
			// end of input stream reached. exit
			return Err(InterpretErr::ExitSig);
		}
		
		let mut line_checker = LineCheck::new();
		let mut tokens = try!(tokenize(&raw_line, &mut line_checker));
		
		tokens.retain(|tok| !tok.is_empty());
		tokens.retain(|tok| match *tok{ TokenType::Delim(..) => false, _ => true });
		
		if line_checker.argc == 0
		{
			return Err(InterpretErr::BlankLine);
		}
		

		let mut cmd_result = InterpretErr::BlankLine;
		{ // scope to sequester borrow caused by iterator
		let mut tokens_iter = tokens.iter();
		
		match tokens_iter.next().unwrap().peek().as_ref()
		{
		"exit" => {
			cmd_result = InterpretErr::ExitSig;
		},
		"format" => {
			let next_tok = tokens_iter.next();
			
			if next_tok.is_none()
			{
				let mut current_fmt = String::with_capacity(80);
				write!(current_fmt, "{}", self.format);
				cmd_result = InterpretErr::CmdSuccess(current_fmt);
			}
			else
			{
				let value = next_tok.unwrap();
				
				let next_fmt = match value.peek().as_ref()
				{
				"s" => ConversionFmt::Short,
				"d" => ConversionFmt::Desc,
				"l" => ConversionFmt::Long,
				_ => return Err(InterpretErr::InvalidState(value.peek().clone())),
				};
				
				self.format = next_fmt;
				cmd_result = InterpretErr::CmdSuccess("Okay.".to_string());
			}
		},
		"help" => {
			cmd_result = InterpretErr::HelpSig;
		},
		keyword @ "input_unit" | keyword @ "output_unit" => {
			let is_input = keyword.starts_with("input");
			let next_tok = tokens_iter.next();
			
			if next_tok.is_none()
			{
				let value = if is_input
				{
					&self.input_unit
				}
				else
				{
					&self.output_unit
				};
				
				cmd_result = InterpretErr::CmdSuccess(
					if value.is_none() { "[not set]".to_string() }
					else { value.clone().unwrap() });
			}
			else
			{
				let mut keyword_string = keyword.to_string();
				let unit_expr_result = parse_unit_expr(&keyword_string);
				
				if unit_expr_result.is_err()
				{
					keyword_string.clear();
					write!(&mut keyword_string, "{}", unit_expr_result.err().unwrap());
					return Err(InterpretErr::InvalidState(keyword_string));
				}
				let unit_expr = unit_expr_result.unwrap();
				
				if unit_expr.alias.is_none() ||
				   unit_expr.prefix != NO_PREFIX ||
				   unit_expr.recall
				{
					return Err(InterpretErr::InvalidState(
							"recall variables may only be a unit name".to_string()));
				}
				
				cmd_result = InterpretErr::CmdSuccess("Okay.".to_string());
			}
		},
		"version" => {
			cmd_result = InterpretErr::VersionSig;
		},
		_ => {
			cmd_result = InterpretErr::ConversionSig;
		},
		};
		
		match cmd_result
		{
		InterpretErr::BlankLine | InterpretErr::ConversionSig => {},
		_ => {
			match tokens_iter.next()
			{
			None => {return Err(cmd_result)},
			Some(tok) => return Err(InterpretErr::UnrecognizedCmd(tok.peek().clone())),
			}
		},
		};
		
		} // end sequestration of iterator
		
		Ok(tokens)
	}
	
	pub fn publish<T>(&mut self, element: &T, mesg: &Option<String>) where T: Display
	{
		match mesg
		{
		&Some(ref text) => { write!(self.output_stream, "{}", text); },
		&None => {},
		};
		
		write!(self.output_stream, "{}", element);
		
		self.output_stream.flush();
	}
	
	pub fn newline(&mut self)
	{
		write!(self.output_stream, "{}",
		       if cfg!(windows) { "\r\n" } else { "\n" }
		);
	}
}