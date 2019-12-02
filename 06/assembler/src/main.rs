use std::fs;
use std::io::{Write, BufWriter};

mod parser;
mod code;

fn main() -> Result<(), std::io::Error> {
    let fres = fs::File::open("PongL.asm")?;
    
    let mut p = parser::Parser::new(fres);
    let mut hack_code = String::new();

    while p.hasMoreComments() {
        p.advance();
        let mut command = String::new();
        match p.commandType() {
            Some(parser::CommandType::A_Command) => {
                let sym = p.symbol();
                // symbol free version
                let num : i32 = sym.parse().unwrap();
                let bin = format!("{:016b}", num);
                command = bin;
            }
            Some(parser::CommandType::C_Command) => {
                command = "111".to_string();
                let comp = code::comp(p.comp());
                if comp.is_none() {
                    println!("Unknown comp command.");
                    continue;
                }
                let compc : String = comp.unwrap().into_iter().collect();
                command.push_str(&compc);

                let dest = code::dest(p.dest());
                if dest.is_none() {
                    println!("Unknown dest command.");
                    continue;
                }
                let destc : String = dest.unwrap().into_iter().collect();
                command.push_str(&destc);

                let jump = code::jump(p.jump());
                if jump.is_none() {
                    println!("Unknown jump command.");
                    continue;
                }
                let jumpc : String = jump.unwrap().into_iter().collect();
                command.push_str(&jumpc);
            }
            Some(parser::CommandType::L_Command) => {
                // TODO
            }
            None => {
                println!("Unknown command.");
                continue;
            }
        }
        if !command.is_empty() {
            hack_code.push_str(&command);
            hack_code.push_str("\r\n");
        }
    }

    print!("{}", hack_code);
    let hack_file_res = fs::File::create("prog.hack")?;

    let mut writer = BufWriter::new(hack_file_res);
    writer.write_all(hack_code.as_bytes())?;
    Ok(())
}
