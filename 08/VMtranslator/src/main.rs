use std::fs;
use std::env;

mod parser;
mod codeWriter;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("not enough arguments");
        return Ok(());
    }
    let file_name = &args[1];

    let f = fs::File::open(file_name)?;
    let mut p = parser::Parser::new(f);

    let mut path = std::path::PathBuf::from(file_name);
    path.set_extension("asm");
    let w_file_name = path.to_string_lossy().to_string();

    let f_w = fs::File::create(&w_file_name)?;
    let mut cw = codeWriter::CodeWriter::new(f_w);

    while p.hasMoreCommands() {
        p.advance();
        match p.commandType() {
            Some(parser::CommandType::C_PUSH) | Some(parser::CommandType::C_POP) => {
                cw.writePushPop(p.commandType().unwrap(), p.arg1(), p.arg2());
            },
            Some(parser::CommandType::C_ARITHMETIC) => {
                cw.writeArithmetic(p.arg1());
            },
            Some(parser::CommandType::C_LABEL) => {
                cw.writeLabel(p.arg1());
            },
            Some(parser::CommandType::C_GOTO) => {
                cw.writeGoto(p.arg1());
            },
            Some(parser::CommandType::C_IF) => {
                cw.writeIf(p.arg1());
            },
            Some(parser::CommandType::C_FUNCTION) => {
                cw.writeFunction(p.arg1(), p.arg2());
            },
            Some(parser::CommandType::C_RETURN) => {
                cw.writeReturn();
            },
            _ => {}
        }
    }

    Ok(())
}
