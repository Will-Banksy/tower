pub mod utils;
/// Contins the parser: Perform lexical and syntactical analysis over a stream of text to validate the syntax and produce a parse tree
pub mod parser;
/// Contains the analyser: Performs semantic analysis over a parse tree to validate its correctness and perhaps perform optimisations and monomorphisation
pub mod analyser;
/// Contains the interpreter - A runtime that takes an AST representation of tower code and executes it according to tower's rules
pub mod interpreter;
/// Contains extension method for operating on a Vec<u8> as the program stack
pub mod stack;
/// Contains the LLVM-based compiler
pub mod compiler;

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