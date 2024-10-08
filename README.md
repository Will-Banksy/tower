# tower

A simplistic stack-based programming language inspired by Factor with a compiler or interpreter (haven't decided yet) written in Rust.

Extremely WIP, the lexer has not been finished yet, and the language design isn't fully decided upon either. This README serves as its documentation and discussion.

<!-- TODO: Update this README - It's out of date with the current state of the language. I started rewriting it a bit but haven't finished (up to Comments) -->

## Concepts

In tower, all operations operate on a stack of values that is shared throughout the whole program.

Functions do not have parameters, since they operate directly on the stack, and so any arguments you want to pass to a function have to be pushed onto the stack.
This does mean that the responsibility of ensuring functions are provided all required arguments falls upon the programmer.

Currently being worked on though is a system of checking *stack effects*, where the exact inputs and outputs of every function are checked to make sure they are correct, much like type checking in most languages - this really requires a proper type system, which is being developed.

## Syntax

Top level tower code is currently composed of comments, function declarations, and struct/enum definitions. Since all functions operate on a stack rather than taking arguments and returning values, tower code is essentially just postfix notation.

UTF-8 is fully supported, so that identifiers can be constructed using the same unicode characters as are allowed in Rust. E.g. `fn ✨ = "sparkle" println ;` is valid but `fn "sparkle" = "✨" println ;` is not.

Whitespace is necessary in many places, such as between `fn` and the function name, but currently all unicode whitespace is allowed and treated equally. So,
```
fn foo {}
```
and
```
fn
foo                         {
	}
```
are interpreted as the same.

### Comments

Comments begin with `#`, and end at the end of the line, bash style. This also allows shebangs.

### (Named) Functions

Function declarations take the form `fn function_name {` followed by the function body, and terminated with a matching `}`.

The function body is a list of literals, words (function calls), and keywords. Literals are like an instruction to push a value on to the stack - E.g. `fn hello = "hello" ;` is a function that simply pushes "hello" onto the stack and then returns. Words are simply the name of a function, which when execution reaches it that function is called - E.g. `fn hello = print_hello ;` is a function that simply calls the `print_hello` function then returns. The only keywords that can appear in a function body are the `{` `}` keywords, denoting the start and end of an anonymous function.

### Anonymous Functions

Anonymous functions are, as their name suggests, functions without a name. They are declared inline inside a function body between a pair of curly brackets `{` `}`. Anything that goes inside a normal/named function body can go inside an anonymous function body.

At parsing, anonymous functions are extracted into a named function with a unique name, and the original declaration turned into a fnptr literal to that uniquely named function, so when execution reaches an anonymous function declaration, a fnptr to the extracted anonymous function is pushed onto the stack.

Anonymous functions are extremely useful for control flow, as they remove the need to declare a named function for each control flow target.

Example:
```
fn main = { "inside an anonymous function" println } dup println call
```
Output:
> &anon_*&lt;unique sequence of numbers and dashes&gt;*\
> inside an anonymous function

## Types

The following types are (or will be) supported:

| Type name | Description                  | Literal example(s)   |
| --------- | -----------------------      | -------------------- |
| str       | UTF-8 string                 | `"hello"`, `"✨"`    |
| bool      | Boolean                      | `true`, `false`      |
| i64       | Signed 64-bit integer        | `-19725`, `1`, `64i` |
| u64       | Unsigned 64-bit integer      | `741u`               |
| f64       | 64-bit floating-point number | `6.9`, `7f`          |
| fnptr     | Function pointer             | `&function_name`     |

TODO: Support different integer sizes

TODO: Also support Algebraic Data Types (ADTs), where the type would be e.g. `bool | i64` and to deconstruct it you use `match` keyword/instruction, which has stack effect e.g. `bool | i64 -> T`, where `T` is the type returned from the anon fns passed to `match` (there may be multiple things returned... how might the stack effect reflect that...? Maybe it just copies the output of the anon fns as it's stack effect... be nice if I could do it in a less special case way tho). Also as we might need named variants, can do something like `None | Some(i64)`

### String Literals

String literals are enclosed in `""` and are UTF-8 compatible. All whitespace in the string literal is included in the final string. Escape sequences similar to in other languages are supported.

Supported escape sequences:
- `\n` - newline
- `\r` - carriage return
- `\t` - tab
- `\0` - null character
- `\\` - single backslash
- `\u{######}` - specified unicode code point, in hex, up to 6 digits

## Control Flow

Branching is done by using the instructions `if` and `ifelse` with fnptrs. Usually, the fnptrs will be anonymous functions.


The `if` instruction expects a fnptr and bool on the stack, and if the bool is `true` then the fnptr is called, otherwise nothing happens.

Example usage:
```
fn main = true { "hello world" println } if
```
Output:
> hello world

The `ifelse` instruction expects two fnptrs and a bool on the stack, and if the bool is true then the first fnptr is called, otherwise the second is.

Example usage:
```
fn main = 1 2 eq { "equal" } { "not equal" } ifelse
```
Output:
> not equal

Loops are done using the `while` instruction, which expects a fnptr and a bool on the stack, much like `if`. `while` pops the two arguments off the stack, and if the bool is true, the fnptr gets called. After that, `while` expects another bool on the stack, and if that is true, the fnptr gets called again.

Example usage:
```
fn main = 0 true { 1 add dup dup print 10 ne " " print } while
```
Output:
> 1 2 3 4 5 6 7 8 9 10

## Standard Library

No standard library has been finalised yet. Currently, there are a few functions such as `print`, `dup`, `add`, etc. that are implemented in the interpreter, known internally as *instructions*.

Ideally I'd like to have a way to define the standard library and whole language inside of itself, but most basic operations such as arithmetic and stack manipulation will for now need to be defined in the compiler/interpreter (function argument binding would be a way to move stack manipulation to be writable in pure tower). C/rust interop and low level functionality (inline IR?) might be a way forward, if I can think of a way to get that working.

## Dev Notes

### C Interop Notes

Current thinking is to in tower code declare an interface with syntax like below:

```
dynlib "libc.so" decl malloc/i32 -> i8*
dynlib "libc.so" decl fseek/i8* i32 i32 -> i32
```
or
```
extern "libc.so" fn malloc/i32 -> i8*
extern "libc.so" fn fseek/i8* i32 i32 -> i32
```

(The syntax is very much not finalised, the syntax for the function parameters should also probably be integrated into normal tower functions e.g. `fn main/i32 i8** -> i32 =` but maybe take more inspiration from factor's stack effect declarations like `fn shuffle/x y -> y x = y x` (some functions can be untyped? generics? and how is this enforced? This is getting a bit complicated. Maybe just leave it as returning types for now rather than whole stack effect declarations))

And then dynamically load the library with dlopen and call functions with dlsym on posix, LoadLibrary and GetProcAddress on windows (meaning I have to link to the dl library on posix, think on windows should be good as kernel32.dll should be linked automatically) That's if I write an interpreter, a compiler to LLVM IR would be able to avoid that. But then I have to write a compiler.

### Syntax Extensibility

In a similar vein to Factor, perhaps one could define new ways of parsing tower code, like creating new literals... Or simply macros... I can't think of a way of doing this right now, but the keyword `syn` defined like a function might be good.