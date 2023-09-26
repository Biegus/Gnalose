#![allow(dead_code)]

use std::{env, fs, process::exit, time::Instant};
mod compiler;
mod lexer;

mod parser;
mod representation;
mod string_builder;
mod test;
mod token;
mod utility;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    pub struct ArgumentFlags: u32{
        const Verbose= 1<<0;
        const PrintEverything=1<<1;
    }
}

#[derive(Debug, derive_new::new)]
struct Arguments {
    input_file_name: String,
    output_file_name: String,
    flags: ArgumentFlags,
}
#[derive(Debug)]
enum ArgumentParsingError {
    MissingInputFile,
    IncorrectArgument(String),
}
impl ArgumentParsingError {
    pub fn get_descr(&self) -> String {
        return match self {
            Self::MissingInputFile => "No file name provided".to_owned(),
            Self::IncorrectArgument(reason) => "Argument couldn't be parsed: ".to_owned() + reason,
        };
    }
}

fn main() {
    let envs: Vec<String> = env::args().collect();

    let arguments = parse_arguments(&envs[1..]);
    if let Err(err) = arguments {
        println!("err while parsing arguments: {}", err.get_descr());
        return;
    }
    let arguments = arguments.unwrap();

    let code = fs::read_to_string(arguments.input_file_name.trim());

    if let Err(err) = code {
        println!("err while reading from file: {}", err);
        exit(1);
    }
    let code = code.unwrap();

    let result = compile_gnalose_to_c_with_args(code.as_str(), &arguments, |f| println!("{}", f));
    match result {
        Err(err) => {
            println!("err:{}", err);
            exit(1);
        }
        Ok(v) => {
            let res = fs::write(arguments.output_file_name, v);
            if let Err(write_err) = res {
                println!("error while writing to file:{}", write_err);
            }
        }
    }
}

fn parse_arguments<'a>(s: &[String]) -> Result<Arguments, ArgumentParsingError> {
    if s.len() == 0 {
        return Err(ArgumentParsingError::MissingInputFile);
    }
    let mut arguments = ArgumentFlags::empty();
    let mut output = "output.c".to_owned();
    let mut i = 1;
    while i < s.len() {
        if s[i] == "-v" {
            arguments |= ArgumentFlags::Verbose
        }
        if s[i] == "-p" {
            arguments |= ArgumentFlags::PrintEverything
        }
        if s[i] == "-o" {
            output = s
                .get(i + 1)
                .ok_or(ArgumentParsingError::IncorrectArgument(
                    "-o should be followed with output file name".to_owned(),
                ))?
                .clone();

            i += 1;
        }
        i += 1;
    }
    return Ok(Arguments::new(s[0].trim().to_owned(), output, arguments));
}

fn compile_gnalose_to_c_with_args<F>(code: &str, arg: &Arguments, out_func: F) -> Result<String, String>
where
    F: Fn(&str),
{
    let is_verbose = arg.flags.contains(ArgumentFlags::Verbose);
    let is_print_everything = arg.flags.contains(ArgumentFlags::PrintEverything);

    let bef = Instant::now();
    let tokens = lexer::tokenize(code).map_err(|er| format!(" [Lexer] {}", er))?;

    if is_verbose {
        out_func(format!("TOKENIZATION DONE in {} s", bef.elapsed().as_secs_f32()).as_str());
    }
    if is_print_everything {
        out_func(format!("Tokenization Output:\n.{}", token::format_token_collection(tokens.as_slice())).as_str());
    }

    let bef = Instant::now();
    let result = parser::parse_to_repr(&tokens).map_err(|err| format!(" [Parser] {}", err))?;

    if is_verbose {
        out_func(format!("PARSING DONE in {} s", bef.elapsed().as_secs_f32()).as_str());
    }
    if is_print_everything {
        out_func(format!("Parsing Output:\n{}", parser::format_representation(&result)).as_str());
    }

    let bef = Instant::now();
    let result = compiler::compile(&result).map_err(|err| format!(" [Final Compiler]{}", err))?;

    if is_verbose {
        out_func(format!("FINAL COMPILATION STEP DONE IN {} S\n", bef.elapsed().as_secs_f32()).as_str());
    }
    if is_print_everything {
        out_func(format!("final compilation output:\n{}", result.as_str()).as_str());
    }
    return Ok(result);
}
