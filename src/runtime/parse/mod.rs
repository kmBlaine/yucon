pub mod number;
pub mod unit;

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use ::utils::*;
use ::runtime::parse::number::{NumberExpr, parse_number_expr};
use ::runtime::parse::unit::{UnitExpr, parse_unit_expr};

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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result
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
                                      recall: false,
                                      tag: None };
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
                },
            };
        }
    }

    Ok(ConvPrimitive { input_vals: value_exprs,
                       input_unit: unit_in_expr,
                       output_units: unit_out_exprs })
}