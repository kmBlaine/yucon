
use ::runtime::parse::ExprParseError;
use ::utils::*;

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

        if !self.valid
        {
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
    let mut tokens: Vec<TokenType> = try!(tokenize(token, &mut number_check));
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