//! Brainfuck interpreter with a virtual machine
//! typical with 30000 cells and doesn't allow to extend
//!
//! Running brainfuck program on the virtual machine

use bft_types::{BFCharCmdName, BFCharInfo, BFProgram};
use std::default::Default;
use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::ops::Drop;

/// Brainfuck IO error with command causes that error
#[derive(Debug)]
pub struct BFVirtualMachineIOErr {
    err: io::Error,
    cmd: BFCharInfo,
}

/// Definition of brainfuck virtual machine errors
#[derive(Debug)]
pub enum BFVmErr {
    /// brainfuck virtual machine's head's falling off either side of tapes
    /// ie. before zero or after the last cell in a non auto-extending case
    /// constructed with the instruction which caused the problem
    HeadInvalidPositionErr(BFCharInfo),

    /// IO error when reading/writing brainfuck virtual machine's cell
    IOErr(BFVirtualMachineIOErr),

    /// brainfuck program square bracket unmatch error
    BracketPairErr(BFCharInfo),
}

impl fmt::Display for BFVmErr {
    /// print brainfuck virtual machine error in human readable format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::HeadInvalidPositionErr(e) => {
                write!(f, "Head falling off edge by {}", e)
            }
            Self::IOErr(e) => {
                write!(f, "{} by {}", e.err, e.cmd)
            }
            Self::BracketPairErr(e) => {
                write!(f, "Unmatched square bracket by {}", e)
            }
        }
    }
}

/// convert std::io::Error into BFVmErr
impl From<BFVirtualMachineIOErr> for BFVmErr {
    fn from(error: BFVirtualMachineIOErr) -> Self {
        BFVmErr::IOErr(error)
    }
}

/// convert BFVmErr to std:error::Error
impl Error for BFVmErr {}

/// Detect and print a newline if non at the end of Brainfuck output
pub struct BFPrintNewLine<'a> {
    tail: u8,
    writer: &'a mut dyn Write,
}

impl<'a> BFPrintNewLine<'a> {
    /// create a new print newline descriptor
    pub fn new(tail: u8, writer: &'a mut dyn Write) -> Self {
        Self { tail, writer }
    }

    /// print out newline if necessary
    pub fn print_newline(&mut self) {
        if self.tail != 0xA {
            if let Some(e) = self.writer.write_all(&[0xA]).err() {
                println!("{}", e);
            }
        }
    }
}

impl<'a> Drop for BFPrintNewLine<'a> {
    /// destructor for BFPrintNewLine, print newline automatically at the
    /// end of lifecycle of the BFPrintNewLine
    fn drop(&mut self) {
        self.print_newline();
    }
}

/// a list of method to handle the brainfuck virtual machine cells
pub trait CellKind {
    /// increment value in current cell by one
    fn increment(&mut self);

    /// decrement value in current cell by one
    fn decrement(&mut self);

    /// read value from current cell
    fn get_value(&mut self) -> u8;

    /// write value into current cell
    fn set_value(&mut self, value: u8);
}

/// Brainfuck virtual machine
///
/// Run simple virtual machine which consists of a tape of cells
/// and a head pointer(default point at cell 0) to the cells.
///
/// # Examples:
///
/// ```
/// use bft_interp::BFVirtualMachine;
/// use bft_types::BFProgram;
/// use std::io;
///
/// let bf_info = BFProgram::new("", "abcd+++123--><,>");
/// let mut bf_vm = BFVirtualMachine::<u8>::new(0, false, &bf_info);
/// bf_vm.move_head_left();
/// let result = bf_vm.interpret(&mut io::stdin(), &mut io::stdout());
/// ```
#[derive(Debug)]
pub struct BFVirtualMachine<'a, T> {
    /// Tape of brainfuck virtual machine
    cells: Vec<T>,

    /// Pointer to head cell
    head: usize,

    /// extendable flag for the tape
    allow_extend: bool,

    /// brainfuck program to be executed on the virtual machine
    program: &'a BFProgram,

    /// current program counter
    program_cnt: usize,
}

impl<'a, T> BFVirtualMachine<'a, T>
where
    T: Default + CellKind,
{
    /// Create a new brainfuck virtual machine
    pub fn new(len: usize, extendable: bool, bf_info: &'a BFProgram) -> Self {
        Self {
            cells: {
                let mut cell_len = 30000;
                if len > 0 {
                    cell_len = len;
                }
                let mut __cells = Vec::<T>::new();
                __cells.resize_with(cell_len, || T::default());
                __cells
            },
            head: 0,
            allow_extend: extendable,
            program: bf_info,
            program_cnt: 0,
        }
    }

    /// Move the head to the left cell, error if falling off low edge
    pub fn move_head_left(&mut self) -> Result<(), BFVmErr> {
        if self.head > 0 {
            self.head -= 1;
            Ok(())
        } else {
            Err(BFVmErr::HeadInvalidPositionErr(
                self.program.instructions()[self.program_cnt],
            ))
        }
    }

    /// Move the head to the right cell, error if falling off high edge
    pub fn move_head_right(&mut self) -> Result<(), BFVmErr> {
        if self.head >= self.cells.len() - 1 {
            if self.allow_extend {
                self.cells.push(T::default());
            } else {
                return Err(BFVmErr::HeadInvalidPositionErr(
                    self.program.instructions()[self.program_cnt],
                ));
            }
        }
        self.head += 1;
        Ok(())
    }

    /// add value at head of tape by 1
    pub fn add_head_by_one(&mut self) {
        self.cells[self.head].increment();
    }

    /// minus value at head of tape by 1
    pub fn minus_head_by_one(&mut self) {
        self.cells[self.head].decrement();
    }

    /// read value from reader to head of tape
    pub fn read_value<R>(&mut self, reader: &mut R) -> Result<(), BFVmErr>
    where
        R: Read,
    {
        let mut buf = vec![0u8; 1];
        println!("Input a value: ");
        reader.read_exact(&mut buf).map_err(|err| {
            BFVmErr::from(BFVirtualMachineIOErr {
                err,
                cmd: self.program.instructions()[self.program_cnt],
            })
        })?;
        self.cells[self.head].set_value(buf[0]);
        Ok(())
    }

    /// write value from head of tape to writer
    pub fn write_value<W>(&mut self, writer: &mut W) -> Result<(), BFVmErr>
    where
        W: Write,
    {
        writer
            .write_all(&[self.cells[self.head].get_value()])
            .map_err(|err| {
                BFVmErr::from(BFVirtualMachineIOErr {
                    err,
                    cmd: self.program.instructions()[self.program_cnt],
                })
            })
    }

    /// enter into loop mode in brainfuck program
    pub fn start_loop(&mut self, idx: Option<usize>) -> Result<(), BFVmErr> {
        if self.cells[self.head].get_value() == 0 {
            match idx {
                Some(r) => self.program_cnt = r,
                None => {
                    return Err(BFVmErr::BracketPairErr(
                        self.program.instructions()[self.program_cnt],
                    ))
                }
            }
        }
        Ok(())
    }

    /// exit loop mode in brainfuck program
    pub fn stop_loop(&mut self, idx: Option<usize>) -> Result<(), BFVmErr> {
        if self.cells[self.head].get_value() > 0 {
            match idx {
                Some(r) => self.program_cnt = r,
                None => {
                    return Err(BFVmErr::BracketPairErr(
                        self.program.instructions()[self.program_cnt],
                    ))
                }
            }
        }
        Ok(())
    }

    /// run brainfuck program on the virtual machine
    pub fn interpret(
        &mut self,
        reader: &mut impl Read,
        writer: &mut impl Write,
    ) -> Result<(), BFVmErr> {
        let mut tail: u8 = 0;
        let cmd_len = self.program.instructions().len();
        while self.program_cnt < cmd_len {
            match self.program.instructions()[self.program_cnt].get_raw() {
                BFCharCmdName::PointerIncrement => self.move_head_right()?,
                BFCharCmdName::PointerDecrement => self.move_head_left()?,
                BFCharCmdName::DataIncrement => self.add_head_by_one(),
                BFCharCmdName::DataDecrement => self.minus_head_by_one(),
                BFCharCmdName::DataOutput => {
                    tail = self.cells[self.head].get_value();
                    self.write_value(writer)?;
                }
                BFCharCmdName::DataInput => self.read_value(reader)?,
                BFCharCmdName::LoopStart(r) => self.start_loop(r)?,
                BFCharCmdName::LoopTerminate(r) => self.stop_loop(r)?,
            }
            self.program_cnt += 1;
        }
        BFPrintNewLine::new(tail, writer);
        Ok(())
    }
}

impl CellKind for u8 {
    /// increment value in current cell by one
    fn increment(&mut self) {
        *self = self.wrapping_add(1);
    }

    /// decrement value in current cell by one
    fn decrement(&mut self) {
        *self = self.wrapping_sub(1);
    }

    /// read value from current cell
    fn get_value(&mut self) -> u8 {
        *self
    }

    /// write value into current cell
    fn set_value(&mut self, value: u8) {
        *self = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_head() {
        let bf_info = BFProgram::new("", "hello++--,><>");
        let mut vm = BFVirtualMachine::<u8>::new(2, false, &bf_info);
        assert!(vm.move_head_left().is_err());
        assert!(vm.move_head_right().is_ok());
        assert!(vm.move_head_right().is_err());
    }

    #[test]
    fn test_move_head_extend() {
        let bf_info = BFProgram::new("", "hello++--,><>");
        let mut vm = BFVirtualMachine::<u8>::new(2, true, &bf_info);
        assert!(vm.move_head_left().is_err());
        assert!(vm.move_head_right().is_ok());
        assert!(vm.move_head_right().is_ok());
    }

    #[test]
    fn test_rw_cells() {
        use std::io::Cursor;
        let mut r_buf = Cursor::new(vec![1, 2, 3, 4, 5]);
        let mut w_buf = Cursor::new(vec![0, 0, 0, 0, 0]);
        let bf_info = BFProgram::new("", "hello++--,><>");
        let mut vm = BFVirtualMachine::<u8>::new(5, false, &bf_info);

        assert_ne!(r_buf.get_ref(), w_buf.get_ref());
        for __i in 0..5 {
            vm.read_value(&mut r_buf).unwrap();
            vm.write_value(&mut w_buf).unwrap();
        }
        assert_eq!(r_buf.get_ref(), w_buf.get_ref());
    }

    #[test]
    fn test_inc_dec_data() {
        use std::io::Cursor;
        let mut r_buf = Cursor::new(vec![1, 2, 3, 4, 5]);
        let mut w_buf = Cursor::new(vec![0, 0, 0, 0, 0]);
        let bf_info = BFProgram::new("", "hello++--,><>");
        let mut vm = BFVirtualMachine::<u8>::new(5, false, &bf_info);

        for __i in 0..5 {
            vm.read_value(&mut r_buf).unwrap();
            vm.add_head_by_one();
            vm.write_value(&mut w_buf).unwrap();
        }
        assert_eq!(w_buf.get_ref(), &Vec::<u8>::from([2, 3, 4, 5, 6]));

        r_buf.set_position(0);
        w_buf.set_position(0);
        for __i in 0..5 {
            vm.read_value(&mut r_buf).unwrap();
            vm.minus_head_by_one();
            vm.write_value(&mut w_buf).unwrap();
        }
        assert_eq!(w_buf.get_ref(), &Vec::<u8>::from([0, 1, 2, 3, 4]));
    }
}
