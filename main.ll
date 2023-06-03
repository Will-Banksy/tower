; @str = private constant [14 x i8] c"Hello, world!\00"
; @fmt_str = private constant [4 x i8] c"%i\0A\00"

@read_mode_str = private constant [2 x i8] c"r\00"
@fmt_i32_str = private constant [4 x i8] c"%d\0A\00"
@lf_str = private constant [2 x i8] c"\0A\00"

; @TokenTypeKeyword ...TODO

; TODO: Also decide how conditionals are going to work in tower - we want it simplistic as possible ideally but we might need some more keywords
; Also do I allow overloading or not?
; A way to handle conditionals *and* loops - goto and goif. Have a syntax for declaring labels, and have labels as a value, and then implement conditional goto label (scoped?)
;     e.g. 1:1 "loop" print goto
;          1 1 eq :1 goif "1 != 1" print 1:
; I might scrap the what factor calls "stack effect declarations" cause I'd need a good way to represent it when it might be conditional etc.

; Token struct
; Token {
;     i32 type,
;     i32 subtype
; }

@TokenSize = private constant i32 8

@TokenTypeInvalid = private constant i8 0 ; Invalid, possibly uninitialised token
@TokenTypeKeyword = private constant i8 1 ; fn, goto, goif
@TokenTypeVerb = private constant i8 2 ; Basically a function call, e.g. dup, eq, _print
@TokenTypeLiteral = private constant i8 3 ; Literals are pushed onto the stack immediately, e.g. 1, 73.4, "string", :1
@TokenTypeDeclaration = private constant i8 4 ; A function declaration, including the equals, e.g. main =

@TokenSubtypeKeyword_Fn = private constant i8 0 ; fn
@TokenSubtypeKeyword_Goto = private constant i8 1 ; goto
@TokenSubtypeKeyword_Goif = private constant i8 2 ; goif

@TokenSubtypeVerb_CompDef = private constant i8 0 ; Compiler defined function, e.g. _print
@TokenSubtypeVerb_NativeDef = private constant i8 1 ; Tower defined function

@TokenSubtypeLiteral_Str = private constant i8 0
@TokenSubtypeLiteral_Bool = private constant i8 1
@TokenSubtypeLiteral_Label = private constant i8 2
@TokenSubtypeLiteral_I64 = private constant i8 3
@TokenSubtypeLiteral_F64 = private constant i8 4

@TokenSubtypeDeclaration_CompDef = private constant i8 0 ; Compiler defined function, e.g. _print
@TokenSubtypeDeclaration_NativeDef = private constant i8 1 ; Tower defined function

define i32 @main(i32 %argc, i8** %argv) {
	; %str_ptr = getelementptr [14 x i8], [14 x i8]* @str, i32 0, i32 0
	; %1 = call i32 @puts(i8* %str_ptr)

	; %fmt_str_ptr = getelementptr [4 x i8], [4 x i8]* @fmt_str, i32 0, i32 0
	; %2 = call i32 (i8*, ...) @printf(i8* %fmt_str_ptr, i32 42)

	%filenamePtr = getelementptr inbounds i8*, i8** %argv, i64 1
	%filename = load i8*, i8** %filenamePtr
	%puts_ret_1 = call i32 @puts(i8* %filename)

	%file_contents = call i8* @read_file(i8* %filename)

	%puts_ret_2 = call i32 @puts(i8* %file_contents)

	; %lf_strp = getelementptr [2 x i8], [2 x i8]* @lf_str, i32 0, i32 0
	; %puts_ret_3 = call i32 @puts(i8* %lf_strp)

	%nullptr = call i8* @tokenise(i8* %file_contents)

	call void @free(i8* %file_contents)

	ret i32 0
}

; Produce a stream of tokens... TODO Decide how to implement this
define i8* @tokenise(i8* %code) {
	; TODO: Implement a lexer

	entry:
		%token_size = load i32, i32* @TokenSize
		%tokens_len = mul i32 %token_size, 16
		%tokens = call i8* @malloc(i32 %tokens_len)

		br label %loop

	loop:
		; In LLVM IR, variables can only be defined once - phi defines a variable depending on the previous label
		; - if that was entry, 0, if loopbody, the value of %next_counter
		%counter = phi i32 [ 0, %entry ], [ %next_counter, %loopbody ]
		%condition = icmp slt i32 %counter, 10
		br i1 %condition, label %loopbody, label %exit

	loopbody:
		%next_counter = add i32 %counter, 1
		br label %loop

	exit:
		%fmt_i32_strp = getelementptr [4 x i8], [4 x i8]* @fmt_i32_str, i32 0, i32 0
		%printf_ret = call i32 (i8*, ...) @printf(i8* %fmt_i32_strp, i32 %counter)

		ret i8* null
}

; Opens and reads the file at filename into a buffer of the file size on the heap
define i8* @read_file(i8* %filename) {
	; Open the file
	%read_mode_strp = getelementptr [2 x i8], [2 x i8]* @read_mode_str, i32 0, i32 0
	%file = call i8* @fopen(i8* %filename, i8* %read_mode_strp)

	; Get the size of the file by seeking to the end, getting the position of the file stream pointer, and then rewinding the fstream ptr to the start
	%fseek_ret = call i32 @fseek(i8* %file, i32 0, i32 2)
	%flen = call i32 @ftell(i8* %file)
	call void @rewind(i8* %file)

	; Allocate a buffer on the heap of the file size to store it
	%buf_ptr = call i8* @malloc(i32 %flen)

	; Read the file contents into the buffer
	%flen_i64 = sext i32 %flen to i64
	%num_read = call i64 @fread(i8* %buf_ptr, i64 1, i64 %flen_i64, i8* %file)

	; Close the file
	%fclose_ret = call i32 @fclose(i8* %file)

	ret i8* %buf_ptr
}

declare i32 @puts(i8*) ; string
declare i32 @printf(i8*, ...) ; format string, ...arguments
declare i8* @fopen(i8*, i8*) ; filename, mode -> FILE
declare i32 @fseek(i8*, i32, i32) ; FILE, offset, origin (0 = start of file, 1 = current position in file, 2 = end of file) -> error code
declare i32 @ftell(i8*) ; FILE -> pos of FILE pointer
declare i8* @malloc(i32) ; size -> ptr to allocated memory
declare void @rewind(i8*) ; FILE
declare i64 @fread(i8*, i64, i64, i8*) ; write buffer, element size, num elements, FILE -> num read bytes
declare i32 @fclose(i8*) ; FILE -> error code
declare void @free(i8*) ; ptr to allocated memory
declare i8* @realloc(i8*) ; ptr to allocated memory -> ptr to newly resized allocated memory