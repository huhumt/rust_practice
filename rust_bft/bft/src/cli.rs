//! A brainfuck CLI implementation using clap
pub mod bft_cli_mod {
    use clap::Parser;
    use std::num::NonZeroUsize;
    use std::path::{Path, PathBuf};

    /// Parse input arguments, should require a positional argument
    /// called `PROGRAM` or report an error
    ///
    /// Providing the usual `--help`, `--version` options
    /// In addition with:
    ///     `-c`, or `--cells` with a non-zero numeric argument to allocate
    ///     cell memory for the tape, `--cells 0` should output an error
    ///     `-e`, or `--extensible` which will turn on the auto-extending tape
    ///
    /// # Examples:
    ///
    /// ```
    /// mod cli;
    /// pub use crate::cli::bft_cli_mod::BftCli;
    ///
    /// let cli = BftCli::new();
    /// let program_name = cli.name();
    /// let cell_size = cli.cells_size();
    ///
    /// ```
    #[derive(Debug, Parser)]
    #[command(
        name = "bft",
        author = "Hao Hu",
        version = "1.0.0",
        about = "Brainfuck Application"
    )]
    pub struct BftCli {
        /// must be a brainfuck `PROGRAM`
        #[arg(help = "PROGRAM name", required = true)]
        name: PathBuf,

        /// size of cell memory to allocate
        #[arg(
            short = 'c',
            long = "cells",
            help = "how many cells allocate for tape, must be greater than 0",
            default_value_t = NonZeroUsize::new(30000).unwrap(),
        )]
        cells: NonZeroUsize,

        /// tape extensible flag
        #[arg(
            short = 'e',
            long = "extensible",
            help = "whether the tape is extensible",
            default_value_t = false
        )]
        allow_extend: bool,
    }

    impl Default for BftCli {
        fn default() -> Self {
            Self::new()
        }
    }

    impl BftCli {
        /// create a new BftCli struct
        pub fn new() -> Self {
            BftCli::parse()
        }

        /// get application name
        pub fn name(&self) -> &Path {
            &self.name
        }

        /// get type size
        pub fn cells_size(&self) -> usize {
            self.cells.get()
        }

        /// get extensible flag
        pub fn cells_extensible(&self) -> bool {
            self.allow_extend
        }
    }
}
