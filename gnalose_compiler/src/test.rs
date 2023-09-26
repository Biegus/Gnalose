#[cfg(test)]
mod test {

    use crate::lexer::*;
    use crate::parser::*;
    use crate::representation::*;
    use crate::token::*;

    use crate::representation::Op::*;

    #[test]
    fn lexer_test_a() {
        let code = "comment/ comment 2/3 haha [hah][[";
        let tokens = tokenize_line(code).unwrap();

        assert_eq!(tokens[0], Token::Comment(String::from("comment")));
        assert_eq!(tokens[1], Token::Comment(String::from(" comment 2")));
        assert_eq!(tokens[2], Token::Literal(3));
        assert_eq!(tokens[3], Token::Name(String::from("haha")));
        assert_eq!(tokens[4], Token::ArrayBracket(ParenthesisSide::Left));
        assert_eq!(tokens[5], Token::Name(String::from("hah")));
        assert_eq!(tokens[6], Token::ArrayBracket(ParenthesisSide::Right));
        assert_eq!(tokens[7], Token::ArrayBracket(ParenthesisSide::Left));
        assert_eq!(tokens[8], Token::ArrayBracket(ParenthesisSide::Left));
    }

    #[test]
    fn lexer_and_parser_test_a() {
        let code = r#"
        read to a
        sub 3 from trash
        read to a
        print a
        undefine trash
        undefine a
        "#;

        let tokens = tokenize(code).unwrap();
        let repr = parse_to_repr(&tokens).unwrap();

        let a: RValue = repr.get_variable("a").unwrap();
        let trash: RValue = repr.get_variable("trash").unwrap();

        assert_eq!(repr.ops.len(), 6);

        assert_eq!(repr.ops[0].op, Define(a));
        assert_eq!(repr.ops[1].op, Define(trash));
        assert_eq!(repr.ops[2].op, Read(VValue::RValue(a)));
        assert_eq!(repr.ops[3].op, Print(AValue::RValue(a)));
        assert_eq!(repr.ops[4].op, Add(AValue::LValue(3), VValue::RValue(trash)));
        assert_eq!(repr.ops[5].op, Print(AValue::RValue(a)));
    }
}
