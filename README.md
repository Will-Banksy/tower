# tower

A simplistic stack-based programming language inspired by Factor with a compiler or interpreter (haven't decided yet) written in LLVM IR.

Why LLVM IR? Mainly as a challenge.

Extremely WIP, the lexer has not been finished yet, but the language design has been somewhat finalised, in my head anyway. This README serves as its documentation.

## Concepts

In tower, all operations operate on a stack of values that is shared throughout the whole program.

Functions do not have parameters, since they operate directly on the stack, and so any arguments you want to pass to a function have to be pushed onto the stack.
This does mean that the responsibility of ensuring functions are provided all required arguments falls upon the programmer.

## Syntax

Top level tower code is composed entirely of comments and the arguably more exciting function declarations.

Comments begin with `#`, and end at the end of the line, bash style.

Function declarations take the form `fn function_name =` followed by the function body, primarily made of what I call *nouns* and *verbs*, but what are essentially just literals and function invocations. Keywords and labels are also present.
All are separated by any ascii whitespace, and function names must be composed of underscores and ASCII alphanumeric characters (but can't start with a number).

A *noun* is a literal value, that, when execution reaches it, is pushed onto the program stack. You can think of it as an instruction to push a value onto the stack. E.g. the function `fn hello = "hello"` will simply push the string "hello" onto the stack and then return.

A *verb* is a function invocation that operates on the current stack. E.g. the function `fn main = 5 5 add` will push the value 5 onto the stack twice and call the `add` function, which pops 2 values off the stack and adds them together, and then pushes the result onto the stack.

It's essentially just postfix notation.

## Types

The following types are (or will be) supported:

| Type name | Description                  | Literal example(s)   |
| --------- | -----------------------      | -------------------- |
| str       | ASCII string                 | `"hello"`            |
| bool      | Boolean                      | `true`, `false`      |
| i64       | Signed 64-bit integer        | `-19725`, `1`, `64i` |
| u64       | Unsigned 64-bit integer      | `741u`               |
| f64       | 64-bit floating-point number | `6.9`, `7f`          |
| label     | Label value                  | `:loop_start`, `:2`  |

## Control Flow

Control flow is done with labels and the keywords `goto` and `goif`.

Labels are much like in other languages, where the syntax is `label_name:`. Label names follow the same rules as function names but can start with a number.
There is a distinction between labels, used for marking a goto destination in the code, and label values, used for indicating which label to go to.

Usage of the `goto` and `goif` keywords is much like normal functions. `goto` pops a label value off of the stack and jumps to the corresponding label, and `goif` pops a label value and a bool off the stack and if that bool is `true` then jumps to the corresponding label.

E.g.:
```
fn loop_5_forever = 0: 5 :0 goto

fn loop_till_10 = 10 0 start: 1 add gte :start goif

# Labels are local to the function they are defined in - the 0: label here does not conflict with the one in the function above and the :0 value here refers to this 0: label
fn looop = 0: :0 goto
```
(where the function `gte` pops 2 values off the stack and pushes `true` if the first is greater than or equal to the second, `false` otherwise)

I'm undecided whether to support the combining of labels and label value literals into one: `start:start`.

## Standard Library

No standard library has been finalised yet. Ideally I'd like to have a way to define the standard library and whole language inside of itself, but most basic operations such as arithmetic and stack manipulation will for now need to be defined in the compiler/interpreter. C interop and low level functionality (inline IR?) might be a way forward, if I can think of a way to get that working.