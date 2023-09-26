use std::num::ParseIntError;

use crate::{token::*, utility::LinedError};

fn is_number(chr: char) -> bool {
    return chr >= '0' && chr <= '9';
}

//assumes no comment
fn tokenize_simple_next(chars: &[char]) -> Result<Option<(&[char], Token)>, (ParseIntError, String)> {
    let mut chars = chars;
    while chars.len() > 0 && chars[0].is_whitespace() {
        chars = &chars[1..]
    }
    if chars.len() == 0 {
        return Ok(None);
    }

    if chars[0] == '[' {
        return Ok(Some((&chars[1..], Token::ArrayBracket(ParenthesisSide::Left))));
    }
    if chars[0] == ']' {
        return Ok(Some((&chars[1..], Token::ArrayBracket(ParenthesisSide::Right))));
    }

    if is_number(chars[0]) {
        let mut i = 1;
        while (i) < chars.len() && is_number(chars[i]) {
            i += 1;
        }
        let number_as_text = chars[0..i].into_iter().collect::<String>();
        let number = number_as_text.parse::<i32>().map_err(|er| (er, number_as_text))?;

        return Ok(Some((&chars[i..], Token::Literal(number))));
    }

    let i = chars
        .iter()
        .position(|&el| el.is_whitespace() || el == '[' || el == ']')
        .unwrap_or(chars.len());

    let result: String = chars[0..i].iter().collect();
    if i + 1 <= chars.len() {
        return Ok(Some((&chars[i..], Token::Name(result))));
    } else {
        return Ok(Some((&[], Token::Name(result))));
    };
}

pub fn tokenize_line(txt: &str) -> Result<Vec<Token>, (ParseIntError, String)> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars: &[char] = &txt.chars().collect::<Vec<char>>();

    loop {
        let comment_index = chars.iter().position(|&el| el == '/');
        if let Some(pos) = comment_index {
            tokens.push(Token::Comment(chars[0..pos].iter().collect()));
            chars = &chars[pos + 1..];
        } else {
            break;
        };
    }

    loop {
        let next = tokenize_simple_next(chars)?;
        match next {
            Some((text, token)) => {
                chars = text;
                tokens.push(token);
            }
            None => {
                break;
            }
        }
    }
    return Ok(tokens);
}

pub fn tokenize(txt: &str) -> Result<Vec<TokenLine>, LinedError<ParseIntError>> {
    let mut vec: Vec<TokenLine> = Vec::new();
    let mut i = 1;

    for line_content in txt.lines().rev() {
        let tokenized =
            tokenize_line(line_content.trim()).map_err(|err| LinedError::new(i, txt.lines().count(), err.1, err.0))?;

        vec.push(TokenLine::new(tokenized, line_content.to_owned()));
        i += 1;
    }
    return Ok(vec);
}
