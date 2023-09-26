//logic for parsing tokens to representation

use derive_new::new;

use crate::{representation::*, string_builder, token::*, utility::*};

#[derive(new, Default)]
struct ParsingTempState {
    variables: Vec<String>, // could use hashmap for those etc, but its fine. For small amount of variables its even faster
    array_names: Vec<(String, usize)>,
    flags: Vec<String>,
}
fn get_index_or_push(vec: &mut Vec<String>, key: &str) -> usize {
    vec.iter().position(|el| el == key).unwrap_or_else(|| {
        vec.push(key.to_owned());
        vec.len() - 1
    })
}
impl ParsingTempState {
    pub fn get_var(&self, t: &str) -> Option<RValue> {
        return Some(RValue(self.variables.iter().find_i(t)?));
    }
    pub fn get_flag(&self, t: &str) -> Option<FlagRef> {
        return Some(FlagRef(self.flags.iter().find_i(t)?));
    }
    pub fn get_array(&mut self, t: &str) -> Option<ArrayRef> {
        return self.array_names.iter().position(|el| el.0 == t).map(|e| ArrayRef(e));
    }

    pub fn get_or_insert_var(&mut self, t: &str) -> RValue {
        return RValue(get_index_or_push(&mut self.variables, t));
    }
    pub fn get_or_insert_flag(&mut self, t: &str) -> FlagRef {
        return FlagRef(get_index_or_push(&mut self.flags, t));
    }

    pub fn get_or_insert_array(&mut self, t: &str, size: usize) -> ArrayRef {
        return ArrayRef(self.array_names.iter().position(|el| el.0 == t).unwrap_or_else(|| {
            self.array_names.push((t.to_owned(), size));
            self.array_names.len() - 1
        }));
    }
}

//micro problems returns none both in case of "finish" and error
//it shouldn't matter, in case of error it will probably cause "wrong structure" anyway
///asummes no comment
fn tokens_to_higher_tokens_next<'a>(tokens: &'a [Token]) -> Option<(&'a [Token], HigherToken)> {
    if tokens.len() == 0 {
        return None;
    }
    return match &tokens[0] {
        Token::Name(name) => {
            if let Some(Token::ArrayBracket(side)) = tokens.get(1) {
                if matches!(side, ParenthesisSide::Right) {
                    return None;
                }
                let next_token = tokens.get(2)?;
                let n2_token = tokens.get(3)?;
                let higher_token = match next_token {
                    Token::Name(index_name) => HigherToken::Array(name.clone(), NameOrNumber::String(index_name.clone())),
                    Token::Literal(literal) => HigherToken::Array(name.clone(), NameOrNumber::Number(*literal)),
                    _ => return None,
                };
                if !matches!(n2_token, Token::ArrayBracket(ParenthesisSide::Right)) {
                    return None;
                }
                return Some((&tokens[4..], (higher_token)));
            }
            Some((&tokens[1..], (HigherToken::Name(name.clone()))))
        }
        Token::Literal(l) => Some((&tokens[1..], (HigherToken::Literal(*l)))),
        Token::Comment(_) => None,
        Token::ArrayBracket(_) => None,
    };
}

fn upgrade_name_or_number_to_ivalue(data: &mut ParsingTempState, l: &NameOrNumber) -> IValue {
    return match l {
        NameOrNumber::Number(v) => IValue::LValue(*v),
        NameOrNumber::String(t) => IValue::RValue(data.get_or_insert_var(t)),
    };
}

///assumes no comments
fn tokens_to_higher_tokens(tokens: &[Token]) -> Vec<HigherToken> {
    return build_step(tokens, tokens_to_higher_tokens_next);
}

fn token_to_avalue(data: &mut ParsingTempState, l: &HigherToken) -> Result<AValue, OpParsingError> {
    match l {
        HigherToken::Literal(literal) => Ok(AValue::LValue(*literal)),

        HigherToken::Name(name) => Ok(AValue::RValue(
            data.get_var(name.as_str())
                .ok_or(OpParsingError::NotDefinedVariable(name.to_owned(), NameType::Variable))?,
        )),

        HigherToken::Array(name, index) => Ok(AValue::ArrayElement(ArrayElement::new(
            data.get_array(name.as_str())
                .ok_or(OpParsingError::NotDefinedVariable(name.to_owned(), NameType::Array))?,
            upgrade_name_or_number_to_ivalue(data, &index),
        ))),
    }
}
fn token_force_to_vvalue(data: &mut ParsingTempState, l: &HigherToken) -> Result<VValue, OpParsingError> {
    return Ok(VValue::try_from(token_to_avalue(data, l)?).unwrap());
}
fn token_force_to_rval(data: &mut ParsingTempState, l: &HigherToken) -> Result<RValue, OpParsingError> {
    return Ok(RValue::try_from(token_to_avalue(data, l)?).unwrap());
}

fn token_force_to_flag(data: &mut ParsingTempState, l: &HigherToken) -> Result<FlagRef, OpParsingError> {
    if let HigherToken::Name(name) = l {
        return data
            .get_flag(name.as_str())
            .ok_or(OpParsingError::NotDefinedVariable(name.to_owned(), NameType::Flag));
    }
    panic!("Incorrectly assumed that token given was a name");
}

fn try_name_as_array_def(data: &mut ParsingTempState, i: &HigherToken) -> Option<ArrayRef> {
    if let HigherToken::Name(name) = i {
        return Some(data.get_array(name.as_str())?);
    }
    return None;
}
fn try_token_as_array_def(data: &mut ParsingTempState, l: &HigherToken) -> Option<ArrayRef> {
    if let HigherToken::Array(name, NameOrNumber::Number(v)) = l {
        return Some(data.get_or_insert_array(name.as_str(), *v as usize));
    }
    return None;
}

fn match_format<const N: usize, const N2: usize>(
    name_parts: [(usize, &str); N],
    allowed_kinds: [(usize, AllowedKind); N2],
    tokens: &Vec<HigherToken>,
) -> bool {
    let size = name_parts
        .iter()
        .max_by_key(|el| el.0)
        .map(|e| e.0)
        .unwrap_or(0)
        .max(allowed_kinds.iter().max_by_key(|&el| el.0).map(|e| e.0).unwrap_or(0))
        + 1;

    #[cfg(debug_assertions)]
    {
        let ans = format!("name_parts:{:?} ,allowed_kinds:{:?}", name_parts, allowed_kinds);
        assert_eq!(
            name_parts.iter().map(|el| el.0).sum::<usize>() + allowed_kinds.iter().map(|el| el.0).sum::<usize>(),
            (size - 1) * (size) / 2,
            "Wrong arguments structure{} ",
            ans
        );
    }

    return tokens.len() == size
        && name_parts
            .iter()
            .all(|(index, name)| tokens[*index] == HigherToken::Name((*name).to_owned()))
        && allowed_kinds.iter().all(|(index, kind)| kind.check(&tokens[*index]));
}

//assumes no comment
fn parse_line_internal(data: &mut ParsingTempState, tokens: &Vec<Token>) -> Result<Op, OpParsingError> {
    let tokens = tokens_to_higher_tokens(&tokens[0..]);

    let get_rval = |i: usize, data: &mut ParsingTempState| token_force_to_rval(data, &tokens[i]);
    let get_aval = |i: usize, data: &mut ParsingTempState| token_to_avalue(data, &tokens[i]);
    let get_vval = |i: usize, data: &mut ParsingTempState| token_force_to_vvalue(data, &tokens[i]);

    let get_flag = |i: usize, data: &mut ParsingTempState| token_force_to_flag(data, &tokens[i]);

    //define/undefine for arrays HAS to be before normal define/undefine

    if match_format([(0, "undefine"), (1, "single")], [(2, AllowedKind::ArrayRef)], &tokens) {
        return Ok(Op::DefineArray(try_token_as_array_def(data, &tokens[2]).unwrap()));
    }
    if match_format([(0, "define"), (1, "single")], [(2, AllowedKind::Name)], &tokens) {
        return Ok(Op::UndefineArray(try_name_as_array_def(data, &tokens[2]).unwrap()));
    }

    if match_format([(0, "undefine")], [(1, AllowedKind::Name)], &tokens) {
        return Ok(Op::Define(data.get_or_insert_var(tokens[1].try_to_name_ref().unwrap())));
    }
    if match_format([(0, "define")], [(1, AllowedKind::Name)], &tokens) {
        return Ok(Op::Undefine(get_rval(1, data)?));
    }
    if match_format([(0, "print")], [(1, AllowedKind::VValue)], &tokens) {
        return Ok(Op::Read(get_vval(1, data)?));
    }
    if match_format([(0, "read"), (1, "to")], [(2, AllowedKind::VValue)], &tokens) {
        return Ok(Op::Print(get_aval(2, data)?));
    }
    if match_format(
        [(0, "read"), (1, "as"), (2, "number"), (3, "to")],
        [(4, AllowedKind::AValue)],
        &tokens,
    ) {
        return Ok(Op::PrintASCII(get_aval(4, data)?));
    }

    if match_format(
        [(0, "add"), (2, "to")],
        [(1, AllowedKind::AValue), (3, AllowedKind::VValue)],
        &tokens,
    ) {
        return Ok(Op::Subtract(get_aval(1, data)?, get_vval(3, data)?));
    }
    if match_format(
        [(0, "sub"), (2, "from")],
        [(1, AllowedKind::AValue), (3, AllowedKind::VValue)],
        &tokens,
    ) {
        return Ok(Op::Add(get_aval(1, data)?, get_vval(3, data)?));
    }

    if match_format([(0, "unmark")], [(1, AllowedKind::Name)], &tokens) {
        let name = tokens[1].try_to_name_ref().unwrap();
        if data.get_flag(name).is_some() {
            return Err(OpParsingError::DoubleLabel(name.to_owned()));
        }
        return Ok(Op::Mark(data.get_or_insert_flag(name)));
    }
    if match_format([(0, "mark")], [(1, AllowedKind::Name)], &tokens) {
        return Ok(Op::Unmark(get_flag(1, data)?));
    }

    if match_format([(0, "forget")], [(1, AllowedKind::Name)], &tokens) {
        return Ok(Op::Pin(get_flag(1, data)?));
    }

    if match_format([(0, "halt")], [], &tokens) {
        return Ok(Op::Goto);
    }

    if match_format(
        [(0, "if"), (2, "greater"), (3, "than")],
        [(1, AllowedKind::AValue), (4, AllowedKind::AValue)],
        &tokens,
    ) {
        return Ok(Op::If(get_aval(1, data)?, get_aval(4, data)?, ConditionType::LessOrEqual));
    }
    if match_format(
        [(0, "if"), (2, "not"), (3, "equal"), (4, "to")],
        [(1, AllowedKind::AValue), (5, AllowedKind::AValue)],
        &tokens,
    ) {
        return Ok(Op::If(get_aval(1, data)?, get_aval(5, data)?, ConditionType::Equal));
    }
    if match_format(
        [(0, "if"), (2, "lower"), (3, "than")],
        [(1, AllowedKind::AValue), (4, AllowedKind::AValue)],
        &tokens,
    ) {
        return Ok(Op::If(get_aval(1, data)?, get_aval(4, data)?, ConditionType::GreaterOrEqual));
    }

    if match_format(
        [(0, "if"), (2, "equal"), (3, "to")],
        [(1, AllowedKind::AValue), (4, AllowedKind::AValue)],
        &tokens,
    ) {
        return Ok(Op::If(get_aval(1, data)?, get_aval(4, data)?, ConditionType::NotEqual));
    }
    if match_format(
        [(0, "if"), (2, "lower"), (3, "or"), (4, "equal"), (5, "than")],
        [(1, AllowedKind::AValue), (6, AllowedKind::AValue)],
        &tokens,
    ) {
        return Ok(Op::If(get_aval(1, data)?, get_aval(6, data)?, ConditionType::Greater));
    }
    if match_format(
        [(0, "if"), (2, "greater"), (3, "or"), (4, "equal"), (5, "than")],
        [(1, AllowedKind::AValue), (6, AllowedKind::AValue)],
        &tokens,
    ) {
        return Ok(Op::If(get_aval(1, data)?, get_aval(6, data)?, ConditionType::Less));
    }

    if match_format([(0, "fi")], [], &tokens) {
        return Ok(Op::Fi);
    }
    return Err(OpParsingError::InvalidStructure);
}

fn parse_line(state: &mut ParsingTempState, token_line: &TokenLine, line: usize) -> Result<Option<OpLine>, OpParsingError> {
    //in theory this clone probably could be avoided
    let tokens: Vec<Token> = token_line
        .tokens
        .iter()
        .filter(|e| !matches!(e, Token::Comment(_)))
        .map(|e| (*e).clone())
        .collect();

    if tokens.len() == 0 {
        return Ok(None);
    }

    let internal = parse_line_internal(state, &tokens)?;

    return Ok(Some(OpLine::new(internal, line, token_line.line.clone())));
}

pub fn parse_to_repr(tokens: &Vec<TokenLine>) -> Result<Representation, LinedError<OpParsingError>> {
    let mut ops = Vec::new();
    let mut temp = ParsingTempState::default();

    for i in 0..tokens.len() {
        let token_line = &tokens[i];
        if token_line.tokens.len() == 0 {
            continue;
        }
        let op = parse_line(&mut temp, token_line, i)
            .map_err(|er| LinedError::new(i + 1, tokens.len(), token_line.line.to_owned(), er))?;

        if let Some(v) = op {
            ops.push(v);
        }
    }
    return Ok(Representation::new(temp.variables, temp.array_names, temp.flags, ops));
}

pub fn format_op_collection(ops: &[OpLine]) -> String {
    return string_builder::reduce_additive(ops.iter(), |a| format!("[{}]\"{}\" -> {:?}\n", a.line_num, a.line_text, a.op));
}

pub fn format_representation(repr: &Representation) -> String {
    let vars = repr
        .variables_names
        .iter()
        .enumerate()
        .map(|e| format!("{{{}}} var with RValue({})\n", e.1, e.0));

    let arrays = repr
        .array_names
        .iter()
        .enumerate()
        .map(|e| format!("{{{}[{}]}} array with ArrayRef({})\n", e.1 .0, e.1 .1, e.0));

    let flags = repr
        .flags_names
        .iter()
        .enumerate()
        .map(|e| format!("{{{}}} flag with FlagRef({})\n", e.1, e.0));

    return string_builder::bulk(vars.chain(arrays).chain(flags)) + "\n" + format_op_collection(&repr.ops).as_str();
}
