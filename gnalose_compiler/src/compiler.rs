use crate::{representation::*, string_builder, utility::LinedError};
#[derive(Debug, derive_more::Display)]
pub enum CompilerError {
    UmmatchedFi,
    UnmathedIf,
}

fn get_includes() -> &'static str {
    return "#include <stdio.h>\n #include <stdbool.h>\n #include <stdlib.h>";
}

fn gen_variable_declaration(decl: &[String]) -> String {
    return string_builder::reduce_additive(decl.iter(), |a| {
        format!("int {}=0; bool {}=false;\n", var_to_pvar(a), pvar_to_switch(&var_to_pvar(a)))
    });
}
fn get_array_declaration(decl: &[(String, usize)]) -> String {
    let mut t = string_builder::Builder::new();
    for var in decl {
        let name = &var.0;
        let size = var.1;
        t.push(
            format!(
                "int {}[{}]={{}}; bool {}=false;\n",
                arr_name_to_pvar(name),
                size,
                pvar_to_switch(&arr_name_to_pvar(name))
            )
            .as_str(),
        );
    }
    return t.collapse();
}

fn get_pre_decl() -> &'static str {
    return r#"int global=0;
void* label=NULL;
int err(char* text)
{
    printf("ABORTED\n:%s",text);
    return 1;
}
#define GOTO if (label!=NULL) goto *label; else return err("nothing to pin")
#define LEAK_CHECK(on_name,normal) if(on_name) {printf("ABORTED\nMemory leaked: %s. Everything should be undefined at the end using \"define\"",normal);return 1;} 
#define ASSERT_ALIVE(bool_name) if(!bool_name) {printf("ABORTED\nTried to use already undefined variable/flag/array");return 1;}
int get(int val)
{
    return val + global;
}

"#;
}

fn get_fake_leak_check(repr: &Representation) -> String {
    let mut builder = Vec::new();
    for var in repr.variables_names.iter() {
        let pvar = var_to_pvar(var);
        let on_v = pvar_to_switch(&pvar);
        string_builder::push(&mut builder, format!("LEAK_CHECK({on_v},\"{var}\");").as_str());
    }
    for var in repr.array_names.iter() {
        let var = &var.0;
        let pvar = arr_name_to_pvar(&var);
        let on_v = pvar_to_switch(&pvar);
        string_builder::push(&mut builder, format!("LEAK_CHECK({on_v},\"{var}\");").as_str());
    }
    for var in repr.flags_names.iter() {
        let pvar = flag_to_pvar(var);
        let on_v = pvar_to_switch(&pvar);
        string_builder::push(&mut builder, format!("LEAK_CHECK({on_v},\"{var}\");").as_str());
    }
    return string_builder::collapse(builder);
}

fn aval_as_txt(avalue: AValue, repr: &Representation) -> String {
    return match avalue {
        AValue::LValue(l) => l.to_string(),
        AValue::RValue(r) => "__".to_owned() + repr.variables_names[r.0].clone().as_str(),
        AValue::ArrayElement(el) => format!(
            "__{}[{}]",
            repr.get_array_name(el.array_ref).as_str(),
            aval_as_txt(el.index.as_avalue(), repr)
        ),
    };
}

fn compile_math_line(a: AValue, b: VValue, plus: bool, repr: &Representation) -> Option<String> {
    let plus_op = if plus { "+" } else { "-" };
    let minus_op = if plus { "-" } else { "+" };

    // addr and temp is cached before so it is not affected by global change
    // addr could be removed if we cached non literal indexes. *probably* gcc compiles it away with optimization enabled
    // counterintuitive element of gnalose: index used to access element is not sheltered from the effect add/sub

    let mut t = format!(
        "{{{assert_a}{assert_b}int temp={v};int* addr=&{nb};global{plus_op}=temp;(*addr){minus_op}=temp;",
        nb = get_pvar_from_repr(b, repr),
        v = aval_as_get(a, repr),
        assert_a = get_alive_assert(a, repr),
        assert_b = get_alive_assert(AValue::from(b), repr)
    );
    if let AValue::RValue(a_id) = a {
        //also safe "a" from effect if it's not literal
        t += format!("{na}{minus_op}=temp;", na = get_pvar_from_repr(VValue::RValue(a_id), repr)).as_str();
    }
    if let AValue::ArrayElement(element) = a {
        t += format!(
            "{na}{minus_op}=temp;",
            na = get_pvar_from_repr(VValue::ArrayElement(element), repr)
        )
        .as_str();
    }
    t += "}";
    return Some(t);
}

fn arr_name_to_pvar(t: &str) -> String {
    return format!("_a_{}", t);
}
fn flag_to_pvar(t: &str) -> String {
    return format!("_f_{}", t);
}
fn var_to_pvar(t: &str) -> String {
    return format!("__{}", t);
}
fn pvar_to_switch(t: &str) -> String {
    return format!("_isOn{}", t);
}

fn get_pvar_from_repr(id: VValue, repr: &Representation) -> String {
    return match id {
        VValue::ArrayElement(el) => {
            format!(
                "{}[{}]",
                arr_name_to_pvar(&repr.get_array_name(el.array_ref)),
                aval_as_get(AValue::from(el.index), repr)
            )
        }
        VValue::RValue(el) => var_to_pvar(&repr.get_variable_name(el)),
    };
}
fn get_switch_from_repr(id: VValue, repr: &Representation) -> String {
    return pvar_to_switch(&get_pvar_from_repr(id, repr));
}
fn get_flag_swith_from_repr(id: FlagRef, repr: &Representation) -> String {
    return pvar_to_switch(&flag_to_pvar(&repr.get_flag_name(id)));
}
fn get_alive_assert(id: AValue, repr: &Representation) -> String {
    match id {
        AValue::LValue(_) => "".into(),
        AValue::ArrayElement(_) | AValue::RValue(_) => {
            format!(
                "ASSERT_ALIVE({on});",
                on = get_switch_from_repr(VValue::try_from(id).unwrap(), repr)
            )
        }
    }
}
fn get_flag_alive_assert(id: FlagRef, repr: &Representation) -> String {
    return format!("ASSERT_ALIVE({on});", on = get_flag_swith_from_repr(id, repr));
}

fn aval_as_get(a: AValue, repr: &Representation) -> String {
    match a {
        AValue::LValue(l) => l.to_string().to_owned(),
        AValue::RValue(_) | AValue::ArrayElement(_) => format!("get({})", aval_as_txt(a, repr)),
    }
}
fn if_to_text(a: AValue, b: AValue, cond: ConditionType, repr: &Representation) -> String {
    let a_name = aval_as_get(a, &repr);
    let b_name = aval_as_get(b, &repr);
    let operator = match cond {
        ConditionType::Equal => "==",
        ConditionType::NotEqual => "!=",
        ConditionType::Greater => ">",
        ConditionType::Less => "<",
        ConditionType::GreaterOrEqual => ">=",
        ConditionType::LessOrEqual => "<=",
    };
    return format!("if({a_name}{operator}{b_name}){{");
}

fn push_builder(txt: &str, builder: &mut String) {
    *builder = (builder.to_owned() + txt) + "\n";
}

#[derive(Debug, derive_new::new)]
struct IfConstruct {
    a: AValue,
    b: AValue,
    cond_type: ConditionType,
    lines: usize,
}
#[derive(Debug, derive_new::new)]
struct CodeBlock {
    code: String,
    last_line: usize,
    if_ending: Option<IfConstruct>,
}
fn try_compile_to_trivial_line(op: &OpLine, repr: &Representation) -> Option<String> {
    match &op.op {
        Op::Define(id) => Some(format!(
            "{n}=-global;{on}=true;",
            n = get_pvar_from_repr(VValue::RValue(*id), &repr),
            on = get_switch_from_repr(VValue::RValue(*id), &repr)
        )),
        Op::DefineArray(id) => {
            let n = arr_name_to_pvar(repr.get_array_name(*id).as_str());
            //TODO: could have global offset for given array for better runtime performance
            return Some(format!(
                "for(int i=0;i<{size};i++)\n{{\n{n}[i]=-global;\n}}\n{on}=true;",
                size = repr.get_array_size(*id),
                on = pvar_to_switch(n.as_str()),
            ));
        }
        Op::Undefine(id) => Some(format!("{on}=false;", on = get_switch_from_repr(VValue::RValue(*id), &repr))),

        Op::UndefineArray(id) => Some(format!(
            "{on}=false;",
            on = pvar_to_switch(arr_name_to_pvar(repr.get_array_name(*id).as_str()).as_str())
        )),
        Op::Read(id) => Some(format!(
            "{assert}scanf(\"%d\",&{n});{n}-=global;",
            assert = get_alive_assert(AValue::from(*id), repr),
            n = get_pvar_from_repr(*id, &repr)
        )),
        Op::Print(val) => Some(format!(
            "{assert}printf(\"%d\\n\",{n});",
            assert = get_alive_assert(*val, repr),
            n = aval_as_get(*val, repr)
        )),
        Op::PrintASCII(val) => Some(format!(
            "{assert}printf(\"%c\\n\",(char){n});",
            assert = get_alive_assert(*val, repr),
            n = aval_as_get(*val, repr)
        )),
        Op::Add(a, b) => compile_math_line(*a, *b, true, repr),
        Op::Subtract(a, b) => compile_math_line(*a, *b, false, repr),
        Op::Mark(flag) => Some(format!("{}:", repr.get_flag_name(*flag))),
        Op::Unmark(flag) => Some(format!(
            "{on}=false;",
            on = pvar_to_switch(flag_to_pvar(repr.get_flag_name(*flag).as_str()).as_str())
        )),
        Op::Pin(flag) => Some(format!(
            "{assert}label=&&{};",
            repr.get_flag_name(*flag),
            assert = get_flag_alive_assert(*flag, repr),
        )),
        Op::Goto => Some("GOTO;".to_owned()),
        Op::If(_, _, _) => None,
        Op::Fi => None,
    }
}
fn compile_internal(ops: &[OpLine], repr: &Representation, line_am: usize) -> Result<CodeBlock, LinedError<CompilerError>> {
    let mut builder: Vec<char> = Vec::new();
    let mut i = 0;
    while i < ops.len() {
        let op_line = &ops[i];
        let trivial = try_compile_to_trivial_line(&op_line, repr).map(|r| r + format!("//{}", op_line.line_text).as_str());

        if let Some(trivial_content) = trivial {
            string_builder::push_line(&mut builder, trivial_content.as_str());
        } else if let Op::Fi = op_line.op {
            let block = compile_internal(&ops[(i + 1)..], repr, line_am)?;
            if let Some(if_content) = block.if_ending {
                let if_text = if_to_text(if_content.a, if_content.b, if_content.cond_type, repr);
                string_builder::push_line(&mut builder, &if_text);
                string_builder::push_line(&mut builder, &block.code);
                string_builder::push_line(&mut builder, "}");
                i += if_content.lines;
            } else {
                return Err(LinedError::new(
                    op_line.line_num,
                    line_am,
                    op_line.line_text.clone(),
                    CompilerError::UmmatchedFi,
                ));
            }
        }
        if let Op::If(a, b, cond) = &op_line.op {
            let constr = IfConstruct::new(*a, *b, *cond, i + 1);
            return Ok(CodeBlock::new(string_builder::collapse(builder), i, Some(constr)));
        }
        i += 1;
    }
    return Ok(CodeBlock::new(string_builder::collapse(builder), i - 1, None));
}

fn get_empty_progam() -> &'static str {
    return "int main(){}";
}
pub fn compile(repr: &Representation) -> Result<String, LinedError<CompilerError>> {
    if repr.ops.len() == 0 {
        return Ok(get_empty_progam().to_owned());
    }

    let lines_count = repr.ops.last().unwrap().line_num;
    let mut builder = String::new();
    builder = (builder + get_includes()) + "\n";
    builder = (builder + get_pre_decl()) + "\n";
    builder += "int main(){\n";
    push_builder(&gen_variable_declaration(&repr.variables_names).as_str(), &mut builder);
    push_builder(&get_array_declaration(&repr.array_names).as_str(), &mut builder);
    push_builder(&get_flag_on_bools(&repr.flags_names).as_str(), &mut builder);

    let result = &compile_internal(&repr.ops, repr, lines_count)?;
    if !result.if_ending.is_none() {
        let rel_op = &repr.ops[result.last_line];
        return Err(LinedError::new(
            rel_op.line_num,
            lines_count,
            rel_op.line_text.clone(),
            CompilerError::UnmathedIf,
        ));
    }
    push_builder(&result.code, &mut builder);
    push_builder(get_fake_leak_check(&repr).as_str(), &mut builder);

    builder += "}";
    return Ok(builder);
}

fn get_flag_on_bools(flags_names: &[String]) -> String {
    let mut builder = string_builder::Builder::new();
    for flag in flags_names {
        builder.push(format!("bool {on}=true;\n", on = pvar_to_switch(&flag_to_pvar(flag))).as_str())
    }
    return builder.collapse();
}
