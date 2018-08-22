/* exec.rs
 * ===
 * Contains the bulk of expression parsing and unit conversion logic.
 * 
 * This file is a part of:
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

use ::unit;
use ::unit::Unit;
use ::unit::UnitDatabase;
use std::rc::Rc;
use std::fmt;
use std::fmt::Display;
use ::parse::SyntaxChecker;
use ::parse::SyntaxError;
use ::parse::TokenType;
use ::parse;
use std::error::Error;

#[derive(Debug)]
pub enum ExprParseError
{
    Syntax(SyntaxError),
    BadPrefix(char),
    EmptyField(String),
}

impl Error for ExprParseError
{
    fn description(&self) -> &str
    {
        match *self
        {
        ExprParseError::Syntax(ref err) => err.description(),
        ExprParseError::BadPrefix(_) => "unknown metric prefix",
        ExprParseError::EmptyField(_) => "field is empty",
        }
    }

    fn cause(&self) -> Option<&Error>
    {
        match *self
        {
        ExprParseError::Syntax(ref err) => Some(err),
        _ => None,
        }
    }
}

impl Display for ExprParseError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match *self
        {
        ExprParseError::Syntax(ref err) => {
            write!(f, "{}", err)
        },
        ExprParseError::BadPrefix(ref ch) => {
            write!(f, "parse error: {}: \'{}\'", self.description(), ch)
        },
        ExprParseError::EmptyField(ref field) => {
            write!(f, "parse error: {} {}", field, self.description())
        },
        }
    }
}

impl From<SyntaxError> for ExprParseError
{
    fn from(err: SyntaxError) -> ExprParseError
    {
        ExprParseError::Syntax(err)
    }
}

#[derive(Debug)]
pub struct GeneralParseError
{
    pub err: ExprParseError,
    pub failed_at: usize, // argument index at which parsing failed
}

impl Error for GeneralParseError
{
    fn description(&self) -> &str
    {
        self.err.description()
    }
    
    fn cause(&self) -> Option<&Error>
    {
        Some(&self.err)
    }
}

impl Display for GeneralParseError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}", self.err)
    }
}

#[derive(Debug)]
pub enum ConversionError
{
    OutOfRange(bool),   // input or output value not a valid f64, false: input
    UnitNotFound(bool), // the unit was not found, false: input
    TypeMismatch,       // the units' types disagree, ie volume into length
}
const INPUT: bool = false;
const OUTPUT: bool = true;
pub const NO_PREFIX: char = '\0';

#[derive(Debug, Copy, Clone)]
pub enum ConversionFmt
{
    Short,
    Desc,
    Long,
}

impl Display for ConversionFmt
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match *self
        {
        ConversionFmt::Short => write!(f, "s: short / value only"),
        ConversionFmt::Desc => write!(f, "d: descriptive / value and output unit"),
        ConversionFmt::Long => write!(f, "l: long / input and output values and units"),
        }
    }
}

fn prefix_as_num(prefix: char) -> Option<f64>
{
    let num: f64 = match prefix
    {
    'Y' => 1.0e24,
    'Z' => 1.0e21,
    'E' => 1.0e18,
    'P' => 1.0e15,
    'T' => 1.0e12,
    'G' => 1.0e9,
    'M' => 1.0e6,
    'k' => 1.0e3,
    'h' => 1.0e2,
    'D' => 1.0e1,
    NO_PREFIX => 1.0,
    'd' => 1.0e-1,
    'c' => 1.0e-2,
    'm' => 1.0e-3,
    'u' => 1.0e-6,
    'n' => 1.0e-9,
    'p' => 1.0e-12,
    'f' => 1.0e-15,
    'a' => 1.0e-18,
    'z' => 1.0e-21,
    'y' => 1.0e-24,
    _   => return None, // default
    };
    
    Some(num)
}

#[derive(Debug)]
pub struct Conversion
{
    from_prefix: char,
    to_prefix: char,
    pub from_alias: String,
    pub to_alias: String,
    pub from: Option<Rc<Unit>>,
    pub to: Option<Rc<Unit>>,
    pub input: f64,
    pub result: Result<f64, ConversionError>,
    pub format: ConversionFmt,
}

impl Conversion
{
    fn new(input_prefix: char, input_alias: String,
        output_prefix: char, output_alias: String, input_val: f64) -> Conversion
    {
        Conversion {
            from_prefix: input_prefix,
            to_prefix: output_prefix,
            from_alias: input_alias,
            to_alias: output_alias,
            from: None,
            to: None,
            input: input_val,
            result: Ok(1.0),
            format: ConversionFmt::Desc,
        }
    }
}

impl Display for Conversion
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self.result
        {
        Ok(ref output) => {
            match self.format
            {
            ConversionFmt::Short => write!(f, "{}", output),
            ConversionFmt::Desc  => {
                let mut prefix = String::with_capacity(1);
                if self.to_prefix != NO_PREFIX
                {
                    prefix.push(self.to_prefix);
                }

                write!(f, "{} {}{}", output, prefix, self.to_alias)
            },
            ConversionFmt::Long  => {
                let mut to_prefix = String::with_capacity(1);
                let mut from_prefix = String::with_capacity(1);

                if self.to_prefix != NO_PREFIX
                {
                    to_prefix.push(self.to_prefix);
                }

                if self.from_prefix != NO_PREFIX
                {
                    from_prefix.push(self.from_prefix);
                }
                write!(f, "{} {}{} = {} {}{}", self.input, from_prefix, self.from_alias,
                    output, to_prefix, self.to_alias)
            },
            }
        },
        Err(ref err) => {
            match err
            {
            &ConversionError::OutOfRange(in_or_out) => {
                write!(f, "conversion error: {} value is out of range",
                    if in_or_out == OUTPUT
                    {
                        "output"
                    }
                    else
                    {
                        "input"
                    })
            },
            &ConversionError::UnitNotFound(in_or_out) => {
                write!(f, "conversion error: no unit called \'{}\' was not found",
                    if in_or_out == OUTPUT
                    {
                        &self.to_alias
                    }
                    else
                    {
                        &self.from_alias
                    })
            },
            &ConversionError::TypeMismatch =>
                write!(f, "conversion error: input and output types differ.\
                          \'{}\' is a {} and \'{}\' is a {}",
                          self.from_alias, self.from.as_ref().unwrap().unit_type,
                          self.to_alias, self.to.as_ref().unwrap().unit_type),
            }
        },
        }
    }
}

enum NumberCheckState
{
    FloatLiteral,
    Semicolon,
    Trailing,
}

struct NumberCheck<'a>
{
    token: &'a String,
    valid: bool,
    state: NumberCheckState,
}

impl<'a> NumberCheck<'a>
{
    fn new(tok: &'a String) -> NumberCheck
    {
        NumberCheck {
            token: tok,
            valid: true,
            state: NumberCheckState::FloatLiteral, 
        }
    }
}

impl<'a> SyntaxChecker for NumberCheck<'a>
{
    fn feed_token(&mut self, token: &str, delim: bool, index: usize) -> bool
    {
        if self.valid
        {
            match self.state
            {
            NumberCheckState::FloatLiteral if !delim => {
                if token.is_empty()
                {
                    self.state = NumberCheckState::Semicolon;
                }
                else if token.parse::<f64>().is_ok()
                {
                    self.state = NumberCheckState::Trailing;
                }
                else
                {
                    self.valid = false;
                }
            },
            NumberCheckState::Semicolon if delim => {
                if token == ";"
                {
                    self.state = NumberCheckState::Trailing;
                }
                else
                {
                    self.valid = false;
                }
            },
            
            NumberCheckState::Trailing => {
                if !token.is_empty()
                {
                    self.valid = false;
                }
            },
            _ => unreachable!("number syntax check reached impossible state"),
            };
        }
        
        self.valid
    }

    fn is_esc(&self, ch: char) -> bool
    {
        false // no escape sequences allowed for numbers
    }

    fn is_comment(&self, ch: char) -> bool
    {
        ch == '#'
    }

    fn is_delim(&self, ch: char) -> bool
    {
        ch == ';'
    }

    fn is_preserved_delim(&self, ch: char) -> bool
    {
        false
    }

    fn esc_char(&self) -> char
    {
        '\\' // dummy. actually no esc sequence.
    }

    fn valid(&self) -> bool
    {
        self.valid
    }

    fn assert_valid(&self, index: usize, more_tokens: bool) -> Result<(), SyntaxError>
    {
        if !more_tokens || !self.valid
        {
            match self.state
            {
            NumberCheckState::FloatLiteral => {
                // reached when receiving a non
                return Err(SyntaxError::Expected(index, "float literal".to_string()));
            },
            NumberCheckState::Semicolon => {
                // not okay to exit without receiving a recall expression
                // not okay to exit without receiving anything
                return Err(SyntaxError::Expected(index, "float literal or recall expression".to_string()));
            },
            _ => (),
            };
        }

        if !self.valid{
            match self.state
            {
            NumberCheckState::Trailing => {
                return Err(SyntaxError::Expected(index, "nothing after value expression".to_string()));
            },
            _ => (),
            };
        }
        
        Ok(())
    }

    fn esc_set(&self) -> bool
    {
        false
    }

    fn set_esc(&mut self, set: bool)
    {
        
    }

    fn reset(&mut self)
    {
        self.valid = true;
        self.state = NumberCheckState::FloatLiteral;
    }
}

pub struct NumberExpr
{
    pub value: f64,
    pub recall: bool,
}

pub fn parse_number_expr(token: &String) -> Result<NumberExpr, ExprParseError>
{
    let mut number_check = NumberCheck::new(token);
    // if the syntax check passed, you know you are either getting a semicolon or a float literal
    let mut tokens: Vec<TokenType> = try!(parse::tokenize(token, &mut number_check));
    tokens.retain(|tok| !tok.is_empty());
    
    if tokens.len() < 1
    {
        return Err(
            ExprParseError::from(
                SyntaxError::Expected(0, "float literal or recall expression".to_string())));
    }
    
    let mut value_expr = NumberExpr {
        value: -1.0,
        recall: false,
    };
    
    for (index, tok) in tokens.drain(..).enumerate()
    {
        if index > 0
        {
            unreachable!("too many tokens in value expression after syntax check");
        }

        match tok
        {
        TokenType::Normal(number) => {
            value_expr.value = match number.parse::<f64>()
            {
            Ok(num) => num,
            Err(err) => {
                unreachable!("float literal cannot be parsed as such after syntax check");
            },
            };
        },
        TokenType::Delim(delim) => {
            if delim == ";"
            {
                value_expr.recall = true;
            }
            else
            {
                unreachable!("illegal value recall character after syntax check");
            }
        },
        };
    }
    
    Ok(value_expr)
}


enum UnitCheckState
{
    NameOrExpr,
    UnderscoreOrColon,
    PrefixOrName,
    Colon,
    Trailing,
}


struct UnitCheck
{
    esc_seq: bool,
    valid: bool,
    state: UnitCheckState,
}

impl UnitCheck
{
    fn new() -> UnitCheck
    {
        UnitCheck {
            esc_seq: false,
            valid: true,
            state: UnitCheckState::NameOrExpr,
        }
    }
}

impl SyntaxChecker for UnitCheck
{
    fn feed_token(&mut self, token: &str, delim: bool, index: usize) -> bool
    {
        if self.valid
        {
            match self.state
            {
            UnitCheckState::NameOrExpr if !delim => {
                if token.is_empty()
                {
                    self.state = UnitCheckState::UnderscoreOrColon;
                }
                else
                {
                    self.state = UnitCheckState::Trailing;
                }
            },
            UnitCheckState::UnderscoreOrColon if delim => {
                if token == "_"
                {
                    self.state = UnitCheckState::PrefixOrName;
                }
                else if token == ":"
                {
                    self.state = UnitCheckState::Trailing;
                }
                else
                {
                    self.valid = false;
                }
            },
            UnitCheckState::PrefixOrName if !delim => {
                if token.is_empty()
                {
                    self.valid = false;
                }
                else if token.len() < 2
                {
                    self.state = UnitCheckState::Colon;
                }
                else
                {
                    self.state = UnitCheckState::Trailing;
                }
            },
            UnitCheckState::Colon if delim => {
                if token == ":"
                {
                    self.state = UnitCheckState::Trailing;
                }
                else
                {
                    self.valid = false;
                }
            },
            UnitCheckState::Trailing => {
                if !token.is_empty()
                {
                    self.valid = false;
                }
            },
            _ => unreachable!("unit expression syntax check reached impossible state"),
            };
        }
        
        self.valid
    }
    
    fn is_esc(&self, ch: char) -> bool
    {
        ch == '\\'
    }
    
    fn is_comment(&self, ch: char) -> bool
    {
        false
    }
    
    fn is_delim(&self, ch: char) -> bool
    {
        ch == '_' ||
        ch == ':'
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
    
    fn assert_valid(&self, index: usize, more_tokens: bool) -> Result<(), SyntaxError>
    {
        if !more_tokens || !self.valid
        {
            match self.state
            {
            UnitCheckState::NameOrExpr | UnitCheckState::UnderscoreOrColon => {
                return Err(SyntaxError::Expected(index,
                        "unit name or recall expression".to_string()));
            },
            UnitCheckState::PrefixOrName | UnitCheckState::Colon => {
                return Err(SyntaxError::Expected(index, 
                        "metric prefix together with unit name / recall expression".to_string()));
            },
            _ => (),
            };
        }
        
        if !self.valid
        {
            match self.state
            {
            UnitCheckState::Trailing => {
                return Err(SyntaxError::Expected(index,
                        "no trailing expressions after unit name".to_string()));
            },
            _ => (),
            };
        }
        
        Ok(())
    }
    
    fn esc_set(&self) -> bool
    {
        self.esc_seq
    }
    
    fn set_esc(&mut self, set: bool)
    {
        self.esc_seq = set;
    }
    
    fn reset(&mut self)
    {
        self.valid = true;
        self.state = UnitCheckState::NameOrExpr;
        self.esc_seq = false;
    }
}

#[derive(Debug,Clone)]
pub struct UnitExpr
{
    pub prefix: char,
    pub alias: Option<String>,
    pub recall: bool,
}

pub fn parse_unit_expr(token: &String) -> Result<UnitExpr, ExprParseError>
{
    let mut expr_checker = UnitCheck::new();
    let mut tokens: Vec<TokenType> = try!(parse::tokenize(token, &mut expr_checker));
    tokens.retain(|tok| !tok.is_empty());
    
    if tokens.len() < 1
    {
        return Err(ExprParseError::from(SyntaxError::Expected(0,
                "metric prefix together with unit name / recall expression".to_string())));
    }
    
    let mut unit_expr = UnitExpr {
        prefix: NO_PREFIX,
        alias: None,
        recall: false,
    };

    let mut tokens_iter = tokens.drain(..);

    match tokens_iter.next().unwrap()
    {
    TokenType::Delim(ref delim) if delim == "_" => {
        let mut alias = tokens_iter.next().unwrap().unwrap();
        let mut new_alias = String::with_capacity(alias.len() - 1);
        let mut alias_iter = alias.chars();
        let prefix = alias_iter.next().unwrap();

        if prefix_as_num(prefix).is_none()
        {
            return Err(ExprParseError::BadPrefix(prefix));
        }
        
        unit_expr.prefix = prefix;
        
        if let Some(trailing) = tokens_iter.next()
        {
            match trailing
            {
            TokenType::Delim(ref colon) if colon == ":" => {
                unit_expr.recall = true;
            },
            _ => unreachable!("illegal delimiter in unit expression after syntax check"),
            };
        }
        else
        {
            for ch in alias_iter
            {
                new_alias.push(ch);
            }
            
            unit_expr.alias = Some(new_alias);
        }
        
        if tokens_iter.next().is_some()
        {
            unreachable!("extra tokens in unit expression after syntax check");
        }
    },
    TokenType::Delim(ref delim) if delim == ":" => {
        unit_expr.recall = true;
    },
    TokenType::Normal(alias) => {
        unit_expr.alias = Some(alias);
    },
    _ => unreachable!("unexpected token begins unit expression"),
    };

    Ok(unit_expr)
}


pub struct ConvPrimitive
{
    pub input_vals: Vec<NumberExpr>,
    pub input_unit: UnitExpr,
    pub output_units: Vec<UnitExpr>,
}

/* Enum for the state matchine of the to_conv_primitive function.
 */
enum ConvPrimState
{
    GetValueExpr,  // get the value expression
    GetMoreValueExpr, // get any additional value expressions
    GetInputExpr,  // get the input unit expression
    GetOutputExpr, // get the output unit expression
    GetMoreOutput, // get any additional output expressions. currently not used
}

/* Takes a line of input that has had its spaces removed as a Vec of TokenType
 * and converts this line into a Number and Unit Exprs for convient use later
 * in the program. Acts as an intermediary to filter out syntax errors before
 * they reach the main conversion routines.
 * 
 * Paramters:
 *   tokens - line tokenized at spaces given as Vec<TokenType>
 * 
 * Returns: Result<>
 *   Ok(ConvPrimitve) - the line converted to expressions
 *   Error(ExprParseError) - error if any occured
 */
pub fn to_conv_primitive(mut tokens: &Vec<TokenType>) -> Result<ConvPrimitive, GeneralParseError>
{
    let mut value_exprs: Vec<NumberExpr> = Vec::new(); //NumberExpr { value: 0.0, recall: false };
    let mut unit_in_expr = UnitExpr { prefix: NO_PREFIX,
                                      alias: None,
                                      recall: false };
    let mut unit_out_exprs: Vec<UnitExpr> = Vec::new();
    
    let mut state = ConvPrimState::GetValueExpr;
    
    for (index, token) in tokens.iter().enumerate()
    {
        let expr = match token
        {
            &TokenType::Delim(_) =>
            {
                unreachable!("conversion primitive generator was given unsanitary input. delimiter detected");
            },
            _ => token.peek(),
        };

        let mut reuse_token = true;

        while reuse_token
        {
            match state
            {
                ConvPrimState::GetValueExpr => {
                    match parse_number_expr(expr)
                    {
                        Ok(new_value_expr) => {
                            value_exprs.push(new_value_expr);
                            state = ConvPrimState::GetMoreValueExpr;
                            reuse_token = false;
                        },
                        Err(expr_parse_err) => {
                            return Err(GeneralParseError { err: expr_parse_err,
                                failed_at: index });
                        }
                    };
                },
                ConvPrimState::GetMoreValueExpr => {
                    match parse_number_expr(expr)
                        {
                            Ok(new_value_expr) => {
                                value_exprs.push(new_value_expr);
                                reuse_token = false;
                            },
                            Err(expr_parse_err) => state = ConvPrimState::GetInputExpr,
                        };
                },
                ConvPrimState::GetInputExpr => {
                    unit_in_expr = match parse_unit_expr(expr)
                    {
                        Ok(new_unit_expr) => new_unit_expr,
                        Err(parse_err) => {
                            return Err(GeneralParseError { err: parse_err,
                                failed_at: index });
                        }
                    };

                    state = ConvPrimState::GetOutputExpr;
                    reuse_token = false;
                },
                ConvPrimState::GetOutputExpr => {
                    match parse_unit_expr(expr)
                    {
                        Ok(new_unit_expr) => unit_out_exprs.push(new_unit_expr),
                        Err(parse_err) => {
                            return Err(GeneralParseError { err: parse_err,
                                failed_at: index });
                        }
                    };
                    state = ConvPrimState::GetMoreOutput;
                    reuse_token = false;
                },
                ConvPrimState::GetMoreOutput => {
                    match parse_unit_expr(expr)
                    {
                        Ok(new_unit_expr) => unit_out_exprs.push(new_unit_expr),
                        Err(parse_err) => {
                            return Err(GeneralParseError { err: parse_err,
                                failed_at: index });
                        }
                    };
                    reuse_token = false;
                }
                _ => {
                    // dummy code for the moment while implemented. needs to be removed
                    unreachable!("too many arguments given to to_conv_primitve()");
                },
            };
        }
    }
    
    Ok(ConvPrimitive { input_vals: value_exprs,
                       input_unit: unit_in_expr,
                       output_units: unit_out_exprs })
}


/* Performs a unit conversion given as an input value, input unit and prefix,
 * and an output unit and prefix. Fetches the units from the given units database
 * A struct conversion is returned allowing the caller to do with it as they
 * please. Note that struct Conversion implements the Display trait and tracks
 * its own validity / error state. This function returns as soon as an error is
 * encountered.
 *
 * Parameters:
 *   - input: the value to be converted
 *   - from_prefix: the single character metric prefix of the input unit
 *   - from: name / alias of the unit to that will be converted
 *   - to_prefix: the single character metric prefix of the output unit
 *   - to: name / alias of the unit to convert to
 *   - units: reference to the database that holds all of the units
 *
 * Stages of Conversion:
 *   1. scale input using prefix and dimensions
 *   2. invert result if necessary
 *   3. change result to base units
 *   4. adjust result to output scale
 *   5. change result to output units
 *   6. invert result if necessary
 *   7. scale result using prefix and dimensions
 */
pub fn convert(input: f64, from_prefix: char, from: String,
    to_prefix: char, to: String, units: &UnitDatabase) -> Conversion
{
    let mut conversion = Conversion::new(from_prefix, from, to_prefix, to, input);

    // if the input value is NaN, INF, or too small
    // Exactly 0 is acceptable however which is_normal() does not account for
    if (!conversion.input.is_normal()) && (conversion.input != 0.0)
    {
        conversion.result = Err(ConversionError::OutOfRange(INPUT));
        return conversion;
    }

    conversion.from = units.query(&conversion.from_alias);
    conversion.to = units.query(&conversion.to_alias);

    if conversion.from.is_none()
    {
        conversion.result = Err(ConversionError::UnitNotFound(INPUT));
    }
    if conversion.to.is_none()
    {
        conversion.result = Err(ConversionError::UnitNotFound(OUTPUT));
    }
    if conversion.result.is_err()
    {
        return conversion;
    }
    
    if conversion.to.as_ref().unwrap().unit_type !=
        conversion.from.as_ref().unwrap().unit_type
    {
        conversion.result = Err(ConversionError::TypeMismatch);
        return conversion;
    }

    // do not initialize yet. we will fetch these values from conversion
    let from_conv_factor: f64;
    let from_zero_point: f64;
    let from_dims: i32;
    let from_is_inverse: bool;
    let to_conv_factor: f64;
    let to_zero_point: f64;
    let to_dims: i32;
    let to_is_inverse: bool;
    {
        // borrow scope for retrieving the unit properties
        // avoids massive method chains on struct Conversion
        let unit_from = conversion.from.as_ref().unwrap();
        from_conv_factor = unit_from.conv_factor;
        from_zero_point = unit_from.zero_point;
        from_dims = unit_from.dimensions as i32;
        from_is_inverse = unit_from.inverse;

        let unit_to = conversion.to.as_ref().unwrap();
        to_conv_factor = unit_to.conv_factor;
        to_zero_point = unit_to.zero_point;
        to_dims = unit_to.dimensions as i32;
        to_is_inverse = unit_to.inverse;
    } // end borrow scope

    // S1
    let mut output_val = conversion.input * prefix_as_num(
        conversion.from_prefix)
        .unwrap().powi(from_dims);

    // S2
    if from_is_inverse
    {
        output_val = 1.0 / output_val;
    }

    output_val *= from_conv_factor; // S3
    output_val += from_zero_point - to_zero_point; // S4
    output_val /= to_conv_factor; // S5

    // S6
    if to_is_inverse
    {
        output_val = 1.0 / output_val;
    }

    // S7
    output_val /= prefix_as_num(conversion.to_prefix).unwrap().powi(to_dims);

    // if the output value is NaN, INF, or too small to properly represent
    // Exactly 0 is acceptable however which is_normal() does not account for
    if (!output_val.is_normal()) && (output_val != 0.0)
    {
        conversion.result = Err(ConversionError::OutOfRange(OUTPUT));
        return conversion;
    }

    conversion.result = Ok(output_val);

    conversion
}

pub fn convert_all(conv_primitive: ConvPrimitive, units: &UnitDatabase) -> Vec<Conversion>
{
    let mut all_conversions = Vec::with_capacity(1);

    for value_expr in conv_primitive.input_vals
    {
        for output_unit in conv_primitive.output_units.iter()
        {
            all_conversions.push(
                convert(value_expr.value,
                        conv_primitive.input_unit.prefix, conv_primitive.input_unit.alias.clone().unwrap(),
                        output_unit.clone().prefix, output_unit.clone().alias.unwrap(),
                        units)
            );
        }
    }
    
    all_conversions
}
