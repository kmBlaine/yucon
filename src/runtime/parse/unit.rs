
use std::vec::Drain;

use ::utils::*;
use ::runtime::parse::ExprParseError;

enum UnitCheckState
{
    NameOrExpr,
    UnderscoreOrColon,
    PrefixOrName,
    Colon,
    FinishOrTag,
    Tag,
    Finish
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
                    self.state = UnitCheckState::FinishOrTag;
                }
            },
            UnitCheckState::UnderscoreOrColon if delim => {
                if token == "_"
                {
                    self.state = UnitCheckState::PrefixOrName;
                }
                else if token == ":"
                {
                    self.state = UnitCheckState::FinishOrTag;
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
                    self.state = UnitCheckState::FinishOrTag;
                }
            },
            UnitCheckState::Colon if delim => {
                if token == ":"
                {
                    self.state = UnitCheckState::FinishOrTag;
                }
                else
                {
                    self.valid = false;
                }
            },
            UnitCheckState::FinishOrTag => {
                if token == "@"
                {
                    self.state = UnitCheckState::Tag
                }
                else if !token.is_empty()
                {
                    self.valid = false;
                }
                // if token is empty, it means we came from Colon. Wait for next
            },
            UnitCheckState::Tag if !delim => {
                if token.is_empty()
                {
                    self.valid = false;
                }
                else
                {
                    self.state = UnitCheckState::Finish;
                }
            },
            UnitCheckState::Finish => {
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
        ch == ':' ||
        ch == '@'
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
            UnitCheckState::Tag => {
                return Err(SyntaxError::Expected(index,
                        "a non-emtpy tag for the unit".to_string()));
            }
            _ => (),
            };
        }

        if !self.valid
        {
            match self.state
            {
            UnitCheckState::FinishOrTag => {
                return Err(SyntaxError::Expected(index,
                        "a tag or nothing at all after unit name / recall expression".to_string()));
            },
            UnitCheckState::Finish => {
                return Err(SyntaxError::Expected(index,
                        "nothing following a tag".to_string()));
            }
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
    pub tag: Option<String>,
}

fn process_alias_or_recall(next_token: Option<TokenType>, unit_expr: &mut UnitExpr, tokens_iter: &mut Drain<TokenType>)
    -> Result<Option<TokenType>, ExprParseError>
{
    match next_token.unwrap()
    {
        TokenType::Normal(alias) => unit_expr.alias = Some(alias),
        TokenType::Delim(ref delim) if delim == ":" => unit_expr.recall = true,
        token @ _ => unreachable!("unexpected token while parsing alias / recall: {:?}", token),
    };

    Ok(tokens_iter.next())
}

fn process_tag(next_token: Option<TokenType>, unit_expr: &mut UnitExpr, tokens_iter: &mut Drain<TokenType>)
    -> Result<Option<TokenType>, ExprParseError>
{
    let more = if next_token.is_some()
    {
        match next_token.unwrap()
        {
            TokenType::Delim(ref delim) if delim == "@" => unit_expr.tag = Some(tokens_iter.next().unwrap().unwrap()),
            token @ _ => unreachable!("unexpected token while parsing tag: {:?}", token),
        };
        tokens_iter.next()
    }
    else
    {
        None
    };

    Ok(more)
}

pub fn parse_unit_expr(token: &String) -> Result<UnitExpr, ExprParseError>
{
    let mut expr_checker = UnitCheck::new();
    let mut tokens: Vec<TokenType> = try!(tokenize(token, &mut expr_checker));
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
        tag: None,
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

        for ch in alias_iter
        {
            new_alias.push(ch);
        }

        let mut iter_result = tokens_iter.next();

        iter_result = try!(process_alias_or_recall(iter_result, &mut unit_expr, &mut tokens_iter));
        iter_result = try!(process_tag(iter_result, &mut unit_expr, &mut tokens_iter));

        if iter_result.is_some()
        {
            unreachable!("extra tokens in unit expression after syntax check");
        }
    },
    TokenType::Delim(ref delim) if delim == ":" => {
        unit_expr.recall = true;
        let iter_result = try!(process_tag(tokens_iter.next(), &mut unit_expr, &mut tokens_iter));
    },
    TokenType::Normal(alias) => {
        unit_expr.alias = Some(alias);
        let iter_result = try!(process_tag(tokens_iter.next(), &mut unit_expr, &mut tokens_iter));
    },
    _ => unreachable!("unexpected token begins unit expression"),
    };

    Ok(unit_expr)
}