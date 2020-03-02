use std::fs;
use std::env;
use std::path;

mod JackTokenizer;
mod CompilationEngine;
mod SymbolTable;
mod VMWriter;

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
        path.set_extension("vm");
        let w_file_name = path.to_string_lossy().to_string();
        let f_w = fs::File::create(&w_file_name)?;
        let mut c = CompilationEngine::CompilationEngine::new(f, f_w, None);
        c.compileClass();
    
        return Ok(());
    }
    else if input.is_dir() {
        let jack_files = input.read_dir()?
            .filter(|d| d.as_ref().unwrap().path().extension().unwrap() == "jack" && !d.as_ref().unwrap().path().ends_with("Sys.vm"))
            .map(|d| d.unwrap().path())
            .collect::<Vec<path::PathBuf>>();
        
        for p in jack_files {
            print!("{:?}", p);
            let mut path = std::path::PathBuf::from(p.to_string_lossy().to_string());
            path.set_extension("vm");
            let f = fs::File::open(p)?;
            let w_file_name = path.to_string_lossy().to_string();
            let f_w = fs::File::create(&w_file_name)?;
            let mut c = CompilationEngine::CompilationEngine::new(f, f_w, None);
            c.compileClass();
        }
    }
    Ok(())
}
