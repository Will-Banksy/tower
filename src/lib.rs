pub mod str_utils;
/// Contains the lexer: Performs lexical analysis over a string of tower code to produce tokens
pub mod lexer;
/// Contins the parser: Performs parsing or syntactic analysis over a stream of tokens to produce an Abstract Syntax Tree (AST)
pub mod parser;
/// Contains the analyser: Performs semantic analysis over an AST to validate its correctness and perhaps perform optimisations
pub mod analyser;
/// Contains the interpreter - A runtime that takes an AST representation of tower code and executes it according to tower's rules
pub mod interpreter;