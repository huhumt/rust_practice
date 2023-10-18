//! Process brainfuck input programs.
//!
//! Parse and store brainfuck programs into data structures,
//! and provide interface to be consumed by brainfuck interpreter

use std::default::Default;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Brainfuck raw command definitions
///
/// There are eight raw commands in brainfuck, each consist of
/// a single character, define them into human readable names.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BFCharCmdName {
    /// '>' character, increment the data pointer
    /// to next cell of brainfuck virtual machine to the right
    PointerIncrement,

    /// '<' character, decrement the data pointer
    /// to previous cell of brainfuck virtual machine to the left
    PointerDecrement,

    /// '+' character, increase one to the byte at current data pointer
    DataIncrement,

    /// '-' character, decrease one to the byte at current data pointer
    DataDecrement,

    /// '.' character, output the byte at current data pointer
    DataOutput,

    /// ',' character, accept one byte of input
    /// storing its value in the byte at current data pointer
    DataInput,

    /// '[' character, starting loop, must match exactly with LoopTerminate
    /// with an Option parameter to recorded its matched LoopTerminate index
    /// otherwise will be None
    LoopStart(Option<usize>),

    /// ']' character, loop terminated, must match exactly with LoopStart
    /// with an Option parameter to recorded its matched LoopStart index
    LoopTerminate(Option<usize>),
}

/// Each brainfuck instruction is recorded with line and column information
#[derive(Debug, Copy, Clone)]
pub struct BFCharInfo {
    raw: BFCharCmdName,
    line: usize,
    column: usize,
}

impl fmt::Display for BFCharInfo {
    /// print brainfuck raw command in more human readable format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fmt_info: &str = match self.raw {
            BFCharCmdName::PointerIncrement => "Increment current pointer",
            BFCharCmdName::PointerDecrement => "Decrement current pointer",
            BFCharCmdName::DataIncrement => "Increment current data",
            BFCharCmdName::DataDecrement => "Decrement current data",
            BFCharCmdName::DataOutput => "Print out current data",
            BFCharCmdName::DataInput => "Type into current data",
            BFCharCmdName::LoopStart(_) => "Start looping",
            BFCharCmdName::LoopTerminate(_) => "End looping",
        };
        write!(f, "{:>5}:{:<5}> {}", self.line, self.column, fmt_info)
    }
}

impl BFCharInfo {
    /// return BFCharCmdName only
    pub fn get_raw(&self) -> BFCharCmdName {
        self.raw
    }
}

/// Record whole brainfuck program information
///
/// Store all brainfuck raw character information in a vector
/// with filename where the information is loaded from.
/// # Examples:
///
/// ```
/// use bft_types::BFProgram;
///
/// let mut bf_info = BFProgram::new("", "abc<\n123.-");
/// let lens = bf_info.instructions().len();
/// let result = bf_info.match_square_bracket();
///
/// ```
#[derive(Debug, Default)]
pub struct BFProgram {
    filename: PathBuf,
    instructions: Vec<BFCharInfo>,
}

impl BFProgram {
    /// Constructor for BFProgram
    pub fn new(path: impl AsRef<Path>, bf_str: &str) -> Self {
        /// Transfer brainfuck raw character to human readable names
        /// only reserve meaningful brainfuck characters
        fn raw_instruction(input_ch: char) -> Option<BFCharCmdName> {
            match input_ch {
                '>' => Some(BFCharCmdName::PointerIncrement),
                '<' => Some(BFCharCmdName::PointerDecrement),
                '+' => Some(BFCharCmdName::DataIncrement),
                '-' => Some(BFCharCmdName::DataDecrement),
                '.' => Some(BFCharCmdName::DataOutput),
                ',' => Some(BFCharCmdName::DataInput),
                '[' => Some(BFCharCmdName::LoopStart(None)),
                ']' => Some(BFCharCmdName::LoopTerminate(None)),
                _ => None,
            }
        }

        let mut bf_char_info = Vec::<BFCharInfo>::new();
        let mut line_number = 1;

        let mut index = 0;
        let mut open_square_bracket_vec = Vec::<usize>::new();

        for line in bf_str.lines() {
            let mut col_number = 1;
            for ch in line.chars() {
                if let Some(r) = raw_instruction(ch) {
                    bf_char_info.push(BFCharInfo {
                        raw: r,
                        line: line_number,
                        column: col_number,
                    });

                    if ch == '[' {
                        open_square_bracket_vec.push(index);
                    } else if ch == ']' {
                        if let Some(r) = open_square_bracket_vec.pop() {
                            bf_char_info[index].raw = BFCharCmdName::LoopTerminate(Some(r));
                            bf_char_info[r].raw = BFCharCmdName::LoopStart(Some(index));
                        }
                    }

                    index += 1;
                }
                col_number += 1;
            }
            line_number += 1;
        }

        BFProgram {
            filename: path.as_ref().to_path_buf(),
            instructions: bf_char_info,
        }
    }

    /// Constructor for BFProgram from file, may encounter IO error
    pub fn from_file(filename: impl AsRef<Path>) -> std::io::Result<BFProgram> {
        let path = filename.as_ref();
        Ok(BFProgram::new(path, &fs::read_to_string(path)?))
    }

    /// A reference of brainfuck instructions
    pub fn instructions(&self) -> &[BFCharInfo] {
        &self.instructions
    }

    /// Brainfuck program must be balanced of open and close square-bracket
    /// Check to ensure it's a valid brainfuck program
    pub fn match_square_bracket(&mut self) -> Result<(), Box<dyn Error>> {
        for s in self.instructions() {
            match s.raw {
                BFCharCmdName::LoopStart(r) => {
                    if r.is_none() {
                        return Err(format!(
                            "bft: Error in input file {}, no open bracket \
                            found matching at line {} column {}",
                            self.filename.display(),
                            s.line,
                            s.column
                        )
                        .into());
                    }
                }
                BFCharCmdName::LoopTerminate(r) => {
                    if r.is_none() {
                        return Err(format!(
                            "bft: Error in input file {}, no close bracket \
                            found matching at line {} column {}",
                            self.filename.display(),
                            s.line,
                            s.column
                        )
                        .into());
                    }
                }
                _ => (),
            }
        }

        Ok(())
    }

    /// Print out BFProgram data
    pub fn print_info(&mut self) {
        for cur_cmd in self.instructions() {
            println!("{}: {}", self.filename.display(), cur_cmd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_col_bf_char() {
        let mut bf_info = BFProgram::new("", "test001++  hello --");
        assert!(bf_info.match_square_bracket().is_ok());

        let read_instructions = bf_info.instructions();
        let target_instructions: Vec<BFCharInfo> = vec![
            BFCharInfo {
                raw: BFCharCmdName::DataIncrement,
                line: 1,
                column: 8,
            },
            BFCharInfo {
                raw: BFCharCmdName::DataIncrement,
                line: 1,
                column: 9,
            },
            BFCharInfo {
                raw: BFCharCmdName::DataDecrement,
                line: 1,
                column: 18,
            },
            BFCharInfo {
                raw: BFCharCmdName::DataDecrement,
                line: 1,
                column: 19,
            },
        ];

        for (i, x) in read_instructions.iter().enumerate() {
            let t: &BFCharInfo = &target_instructions[i];
            assert_eq!(x.raw, t.raw);
            assert_eq!(x.line, t.line);
            assert_eq!(x.column, t.column);
        }
    }

    #[test]
    fn test_match_square_bracket() {
        let mut bf_info = BFProgram::new("", "test001++  hello --[>,<+>--,[]");
        assert!(bf_info.match_square_bracket().is_err());
    }
}
