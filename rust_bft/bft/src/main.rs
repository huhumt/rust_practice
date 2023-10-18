//! Brainfuck application
//!
//! Parsing brainfuck instructions from files then running the program
//! on the brainfuck interpreter with a virtual machine.

use bft_interp::BFVirtualMachine;
use bft_types::BFProgram;
use std::error::Error;
use std::io;
use std::process::ExitCode;

mod cli;
pub use crate::cli::bft_cli_mod::BftCli;

/// run bft program with cli arguments
fn bft_run(cli: &BftCli) -> Result<(), Box<dyn Error>> {
    let mut bf_info = BFProgram::from_file(cli.name())?;
    bf_info.match_square_bracket()?;

    let size = cli.cells_size();
    let extend = cli.cells_extensible();
    let mut bf_vm = BFVirtualMachine::<u8>::new(size, extend, &bf_info);

    bf_vm.interpret(&mut io::stdin(), &mut io::stdout())?;
    Ok(())
}

/// Main entry for the brainfuck application
fn main() -> ExitCode {
    let cli = BftCli::new();

    if let Some(e) = bft_run(&cli).err() {
        println!("{:#?}", e.to_string());
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}
