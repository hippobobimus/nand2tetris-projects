//! A library containing tooling required to implement a command line Hack assembler program that
//! translates a Hack assembly program into binary Hack machine code.
//!
//! It presents an API with a 'Config' type used to store command line configuration arguments and
//! a 'run' function that carries out the process of translation.
//!
//! Some syntax checking of the Hack assembly instructions takes place, but it is not designed to
//! be exhaustive.  In general the input is assumed to be syntactically correct.

pub use self::config::Config;
pub use self::runner::run;

pub mod config;
pub mod runner;
mod code_translator;
mod error;
mod parser;
mod symbols;
