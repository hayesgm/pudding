extern crate log;

mod wasm;
mod solver;

use std::env;
use std::process;

fn print_usage() -> ! {
    let program = env::args().nth(0).expect("expect 0th arg");
    println!("usage: {} <file.wasm> <func>\n", program);
    process::exit(1)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    match env::args().collect::<Vec<_>>().as_slice() {
        [_, file, func] => {
            let module = wasm::load(file)?;
            let func_num = wasm::func_from_name(&module, &func.as_str())?;
            solver::solve(&module, func_num)?;

            Ok(())
        },
        _ =>
            print_usage()
    }
}
