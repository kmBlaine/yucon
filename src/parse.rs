use std::error;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::fmt::Display;

#[derive(Debug)]
pub enum SyntaxError
{
	Expected(usize, String),
	BadEscSeq(usize, char),
}

impl Error for SyntaxError
{
	fn description(&self) -> &str
	{
		match *self
		{
		SyntaxError::Expected(..) => "expected different token",
		SyntaxError::BadEscSeq(..) => "reached bad escape sequence",
		}
	}
	
	fn cause(&self) -> Option<&Error>
	{
		None
	}
}

impl Display for SyntaxError
{
	fn fmt(&self, f: &mut Formatter) -> fmt::Result
	{
		match *self
		{
		SyntaxError::Expected(ref index, ref msg) => {
			write!(f, "syntax error @ col {}: expected {}", index+1, msg)
		},
		SyntaxError::BadEscSeq(ref index, ref ch) => {
			write!(f, "syntax error @ col {}: bad escape sequence: \\{}", index+1, ch)
		},
		}
	}
}

/* trait SyntaxChecker
 * 
 * Description: this is a generic trait that represents a token-based syntax,
 *   allowing wildly different syntaxes to be handled by the same
 *   tokenization routine and be validated at the time of tokenization. See
 *   'fn tokenize' for more details.
 *
 * Usage:
 *   fn feed_token(&mut self, &str, usize) -> bool
 *     Checks the next token against syntax rules. Returns true if the syntax
 *     check encountered no errors. Returns false otherwise.
 *
 *     - &mut self  : mutable reference to the struct implementing this trait.
 *                    mutation may not be required depending on implementation,
 *                    it is best to have the option.
 *     - &str       : token to check
 *     - bool       : indicates whether a delimiter or token was fed. true for
 *                    delimiter, false for token.
 *     - usize      : index where tokenization left off
 *
 *   fn is_esc(&self, char) -> bool
 *     Checks if the given char was this syntax's escape sequence char. Returns
 *     true if it was and false otherwise.
 *
 *     - char       : character to check
 *
 *   fn is_comment(&self, char) -> bool
 *     Checks if the given char was this syntax's comment char. Returns
 *     true if it was and false otherwise.
 *
 *     - char       : character to check
 *
 *   fn is_delim(&self, char) -> bool
 *     Checks if the given char was a delimiter used by this syntax. Returns
 *     true if it was and false otherwise.
 *
 *     - char       : character to check
 *
 *   fn valid(&self) -> bool
 *     Returns the state of the syntax checker. True if no rules have been
 *     violated, false otherwise.
 *
 *   fn asser_valid(&self, usize, bool) -> Result<(), SyntaxError>
 *     Asserts that the syntax is valid by returning nothing if it is and a
 *     SyntaxError if it is invalid. Meant for ergonomic use with try! macro.
 *     Configurable to check whether the syntax is in a valid exit state where
 *     no more tokens will be received or whether the syntax is in valid
 *     progressive state where we are expecting more tokens.
 *
 *     - usize      : index where tokenization left off
 *     - bool       : indicates whether we are expecting more tokens or not
 *                    True for expecting more, False for no more tokens.
 *
 *   fn esc_set(&self) -> bool
 *     Returns true if the next character will be escaped, false otherwise.
 *     Optional trait. Provided for cooperation with the tokenization routine
 *     if there are special rules concerning how escape sequences are determined.
 *
 *   fn set_esc(&mut self, bool)
 *     Attempts to make it so next character will be escaped.
 *     Optional trait. Provided for cooperation with the tokenization routine
 *     if there are special rules concerning how escape sequences are determined.
 *
 *   fn reset(&mut self)
 *     Resets this syntax to its default state.
 */
pub trait SyntaxChecker
{
	fn feed_token(&mut self, token: &str, delim: bool, index: usize) -> bool;
	fn is_esc(&self, ch: char) -> bool;
	fn is_comment(&self, ch: char) -> bool;
	fn is_delim(&self, ch: char) -> bool;
	fn is_preserved_delim(&self, ch: char) -> bool;
	fn esc_char(&self) -> char;
	fn valid(&self) -> bool;
	fn assert_valid(&self, index: usize, more_tokens: bool) -> Result<(), SyntaxError>;
	fn esc_set(&self) -> bool;
	fn set_esc(&mut self, set: bool);
	fn reset(&mut self);
}

const DELIM: bool = true; // constant for indicated delimiter to SyntaxChecker trait

/* enum TokenType
 *
 * Description: wrapper for tokens that denotes them as either delimiters or
 *   normal tokens. used to disambiguate tokens after tokenization which could
 *   otherwise be confused.
 *   Ex. "aliases = \,,other" ->
 *     Without this wrapper, this line would be tokenized as ',' ',' 'other' and
 *     have no way of telling that the first token was in fact a valid alias not
 *     to be discarded.
 *
 * Contained Types:
 *   - Delim(String)  : wraps a string that is a delimiter
 *   - Normal(String) : wraps a string that is a Normal
 */
#[derive(Debug)]
pub enum TokenType
{
	Delim (String),
	Normal(String),
}

impl TokenType
{
	// Conveniently unwraps the contained string so redundant match lookups are
	// eliminated.
	pub fn unwrap(self) -> String
	{
		match self
		{
			TokenType::Delim(tok)  => return tok,
			TokenType::Normal(tok) => return tok,
		}
	}
	
	// Peeks at the wrapped value. Returns reference to String for convenience
	// when working with borrowed TokenTypes
	pub fn peek(&self) -> &String
	{
		match *self
		{
		TokenType::Delim(ref tok) => tok,
		TokenType::Normal(ref tok) => tok,
		}
	}
	
	// Checks if the contained string is empty so that unwrapping is not
	// necessary
	pub fn is_empty(&self) -> bool
	{
		match *self
		{
			TokenType::Delim(ref tok)  => return tok.is_empty(),
			TokenType::Normal(ref tok) => return tok.is_empty(),
		}
	}
}
/* Attemtps to tokenizes a line according to the syntax described by 'checker'.
 * If the line's syntax is valid, a vector of TokenType wrapped strings will be
 * returned. Otherwise, a SyntaxError will be raised or propogated.
 * 
 * Parameters:
 *   - line    : string of text to be tokenized
 *   - checker : set of syntax rules to tokenize with. must implement
 *               SyntaxChecker trait
 *
 * Important Notes:
 *   - delimiters implicitly separate two tokens even if one of those tokens is
 *     is empty. Thus this routine WILL generate blank tokens to either side of
 *     delimiters as necessary such as when they are chained or when they begin
 *     or end a line. It is the CALLER's reponsibility to deal with blank tokens
 *   - this routine discards comments ENTIRELY. neither the comment delimiter
 *     nor the comment will be present in the result vector. this is because
 *     by definition comments are semantically meaningless.
 */
pub fn tokenize<S: SyntaxChecker>(line: &str, checker: &mut S) -> Result<Vec<TokenType>, SyntaxError>
{
	if line.is_empty()
	{
		let mut tokens = Vec::with_capacity(1);
		tokens.push(TokenType::Normal(String::new()));
		return Ok(tokens);
	}
	let mut buffer = String::with_capacity(line.len()); // biggest token is possible is the line unmodified
	let mut tokens = Vec::with_capacity(5); // unit properties contain at least 3 tokens, CommonName 5. avoids excessive reallocation
	let mut delim_pushed = false;
	let mut last: usize = 0;
	let mut last_ch: char = '\0';

	for (index, ch) in line.chars().enumerate()
	{
		if checker.is_esc(ch) && !checker.esc_set()
		{
			checker.set_esc(true);
		}
		else if checker.esc_set()
		{
			if checker.is_delim(ch) || checker.is_esc(ch) || checker.is_comment(ch)
			{
				buffer.push(ch);
				checker.set_esc(false);
				delim_pushed = false;
			}
			else if checker.is_preserved_delim(ch)
			{
				buffer.push(checker.esc_char());
				buffer.push(ch);
				checker.set_esc(false);
				delim_pushed = false;
			}
			else
			{
				last = index;
				last_ch = ch;
				break;
			}
		}
		else if checker.is_delim(ch)
		{
			let mut new_token = buffer.clone();
			new_token.shrink_to_fit();
			checker.feed_token(&new_token, !DELIM, index);

			tokens.push(TokenType::Normal(new_token));

			buffer.clear();
			buffer.push(ch);

			new_token = buffer.clone();
			new_token.shrink_to_fit();
			checker.feed_token(&new_token, DELIM, index);

			tokens.push(TokenType::Delim(new_token));

			buffer.clear();
			delim_pushed = true;
		}
		else if checker.is_comment(ch)
		{
			let mut new_token = buffer.clone();
			new_token.shrink_to_fit();

			checker.feed_token(&new_token, !DELIM, index);

			tokens.push(TokenType::Normal(new_token));
			try!(checker.assert_valid(index, true));
			return Ok(tokens); // if we reach a comment, immediately exit
		}
		else
		{
			buffer.push(ch);
			delim_pushed = false;
		}

		try!(checker.assert_valid(index, true));
		last = index;
		last_ch = ch;
	}

	if checker.esc_set()
	{
		return Err(SyntaxError::BadEscSeq(last,
						if last_ch == checker.esc_char()
						{
							'\0'
						}
						else
						{
							last_ch
						})
		);
	}

	let mut new_token = String::new();

	if !buffer.is_empty()
	{
		new_token = buffer.clone();
		new_token.shrink_to_fit();
		checker.feed_token(&new_token, !DELIM, last);
		tokens.push(TokenType::Normal(new_token));
	}
	else if delim_pushed
	{
		checker.feed_token(&new_token, !DELIM, last);
		tokens.push(TokenType::Normal(new_token));
	}

	try!(checker.assert_valid(last, false));

	Ok(tokens)
}
