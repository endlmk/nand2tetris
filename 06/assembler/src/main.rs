use std::fs;
use std::io::{Write, Seek, SeekFrom, BufWriter};

mod parser;
mod code;
mod symboltable;

fn main() -> Result<(), std::io::Error> {
    let f = fs::File::open("Pong.asm")?;
    let mut fc = f.try_clone()?;

    let mut p1 = parser::Parser::new(f);
    let mut symbol_table = symboltable::SymbolTable::new();
    let mut addr = 0;

    // Defined Symbols
    symbol_table.addEntry("SP".to_string(), 0);
    symbol_table.addEntry("LCL".to_string(), 1);
    symbol_table.addEntry("ARG".to_string(), 2);
    symbol_table.addEntry("THIS".to_string(), 3);
    symbol_table.addEntry("THAT".to_string(), 4);
    symbol_table.addEntry("R0".to_string(), 0);
    symbol_table.addEntry("R1".to_string(), 1);
    symbol_table.addEntry("R2".to_string(), 2);
    symbol_table.addEntry("R3".to_string(), 3);
    symbol_table.addEntry("R4".to_string(), 4);
    symbol_table.addEntry("R5".to_string(), 5);
    symbol_table.addEntry("R6".to_string(), 6);
    symbol_table.addEntry("R7".to_string(), 7);
    symbol_table.addEntry("R8".to_string(), 8);
    symbol_table.addEntry("R9".to_string(), 9);
    symbol_table.addEntry("R10".to_string(), 10);
    symbol_table.addEntry("R11".to_string(), 11);
    symbol_table.addEntry("R12".to_string(), 12);
    symbol_table.addEntry("R13".to_string(), 13);
    symbol_table.addEntry("R14".to_string(), 14);
    symbol_table.addEntry("R15".to_string(), 15);
    symbol_table.addEntry("SCREEN".to_string(), 16384);
    symbol_table.addEntry("KBD".to_string(), 24576);

    while p1.hasMoreComments() {
        p1.advance();
        match p1.commandType() {
            Some(parser::CommandType::A_Command) => { addr += 1; }
            Some(parser::CommandType::C_Command) => { addr += 1; }
            Some(parser::CommandType::L_Command) => {
                let sym = p1.symbol();
                if !symbol_table.contains(sym) {
                    symbol_table.addEntry(sym.to_string(), addr);
                }
            }
            None => {
                println!("Unknown command.");
                continue;
            }
        }
    }

    fc.seek(SeekFrom::Start(0))?;
    let mut p2 = parser::Parser::new(fc);
    let mut hack_code = String::new();
    let mut v_addr = 16;

    while p2.hasMoreComments() {
        p2.advance();
        let mut command = String::new();
        match p2.commandType() {
            Some(parser::CommandType::A_Command) => {
                let sym = p2.symbol();
                let num_res = sym.parse::<i32>();
                if num_res.is_err() {
                    // Symbol
                    if symbol_table.contains(sym) {
                        // Label or Defined Variable
                        command = format!("{:016b}", symbol_table.getAddress(sym));
                    }
                    else {
                        // New Variable
                        command = format!("{:016b}", v_addr);
                        symbol_table.addEntry(sym.to_string(), v_addr);
                        v_addr += 1;
                    }
                }
                else {
                    let bin = format!("{:016b}", num_res.unwrap());
                    command = bin;
                }
            }
            Some(parser::CommandType::C_Command) => {
                command = "111".to_string();
                let comp = code::comp(p2.comp());
                if comp.is_none() {
                    println!("Unknown comp command.");
                    continue;
                }
                let compc : String = comp.unwrap().into_iter().collect();
                command.push_str(&compc);

                let dest = code::dest(p2.dest());
                if dest.is_none() {
                    println!("Unknown dest command.");
                    continue;
                }
                let destc : String = dest.unwrap().into_iter().collect();
                command.push_str(&destc);

                let jump = code::jump(p2.jump());
                if jump.is_none() {
                    println!("Unknown jump command.");
                    continue;
                }
                let jumpc : String = jump.unwrap().into_iter().collect();
                command.push_str(&jumpc);
            }
            Some(parser::CommandType::L_Command) => {}
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
    let hack_file = fs::File::create("prog.hack")?;

    let mut writer = BufWriter::new(hack_file);
    writer.write_all(hack_code.as_bytes())?;
    Ok(())
}
