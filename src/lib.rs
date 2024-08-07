pub mod utils;
/// Contains the lexer: Performs lexical analysis over a string of tower code to produce tokens
pub mod lexer;
/// Contins the parser: Performs parsing or syntactic analysis over a stream of tokens to produce an Abstract Syntax Tree (AST)
pub mod parser;
pub mod parser_new;
/// Contains the analyser: Performs semantic analysis over an AST to validate its correctness and perhaps perform optimisations and monomorphisation
pub mod analyser;
/// Contains the interpreter - A runtime that takes an AST representation of tower code and executes it according to tower's rules
pub mod interpreter;
/// Contains defined instructions - compiler-defined functions
pub mod instructions;
/// Contains extension method for operating on a Vec<u8> as the program stack
pub mod stack;
/// Contains error types
pub mod error;

/// Acts like the ? operator on ScanResults - Returns early on WithErr and Unrecognised
#[macro_export]
macro_rules! brk {
	($e:expr) => {
		match $e {
			ScanResult::Valid(v) => v,
			ScanResult::WithErr(e) => {
				return ScanResult::WithErr(e)
			}
			ScanResult::Unrecognised => {
				return ScanResult::Unrecognised;
			}
		}
	};
}