use std::fs;
use std::env;
use std::path;

mod parser;
mod codeWriter;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("not enough arguments");
        return Ok(());
    }

    let input = path::Path::new(&args[1]);
    if input.is_file() {
        let file_name = input;

        let f = fs::File::open(file_name)?;
    
        let mut path = std::path::PathBuf::from(file_name);
        path.set_extension("asm");
        let w_file_name = path.to_string_lossy().to_string();
        let f_w = fs::File::create(&w_file_name)?;
        let mut cw = codeWriter::CodeWriter::new(f_w);
        
        proc_translate(&f, file_name.to_str().unwrap(), &mut cw, false);
    
        return Ok(());
    }
    else if input.is_dir() {
        let mut path = std::path::PathBuf::from(input);
        path.set_extension("asm");
        let w_file_name = path.to_string_lossy().to_string();
        let f_w = fs::File::create(&w_file_name)?;
        let mut cw = codeWriter::CodeWriter::new(f_w);

        let mut iter = input.read_dir()?;
        match iter.find(|d| d.as_ref().unwrap().path().ends_with("Sys.vm")) {
            None => {
                println!("Sys.vm is not contained.");
                return Ok(());
            },
            Some(d) => {
                let p = d.unwrap().path();
                let f_name = p.file_name().unwrap().to_string_lossy().to_string();
                let f = fs::File::open(p)?;
                proc_translate(&f, &f_name, &mut cw, true);
            } 
        }

        for test in input.read_dir()? {
            print!("{:?}", test.unwrap().path().extension().unwrap() == "vm");
        }

        let vm_files = input.read_dir()?
            .filter(|d| d.as_ref().unwrap().path().extension().unwrap() == "vm" && !d.as_ref().unwrap().path().ends_with("Sys.vm"))
            .map(|d| d.unwrap().path())
            .collect::<Vec<path::PathBuf>>();
        
        for p in vm_files {
            let f_name = p.file_name().unwrap().to_string_lossy().to_string();
            let f = fs::File::open(p)?;
            proc_translate(&f, &f_name, &mut cw, false);
        }
    }
    
    Ok(())
}

fn proc_translate(f: &fs::File, f_name: &str, cw: &mut codeWriter::CodeWriter<fs::File>, is_bootstrap: bool) {
    let mut p = parser::Parser::new(f);
    cw.setFileName(f_name);
    if is_bootstrap {
        cw.writeInit();
    }

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
            Some(parser::CommandType::C_CALL) => {
                cw.writeCall(p.arg1(), p.arg2());
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

}
