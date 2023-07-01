; @str = private constant [14 x i8] c"Hello, world!\00"
; @fmt_str = private constant [4 x i8] c"%i\0A\00"

@read_mode_str = private constant [2 x i8] c"r\00"
@fmt_i32_str = private constant [4 x i8] c"%d\0A\00"
@lf_str = private constant [2 x i8] c"\0A\00" ; "\n\0"
@ws_chars_str = private constant [4 x i8] c"\0A\0D\09\20" ; "\n\r\t "
@equals_sign = private constant i8 61 ; '='

@dbg_lex_kwrd_str = private constant [28 x i8] c"Keyword found at index: %d\0A\00"
@dbg_tokens_len_str = private constant [19 x i8] c"Tokens length: %d\0A\00"

@keyword_fn_str = private constant [3 x i8] c"fn "

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

@token_size = private constant i32 8

; @TokenTypeInvalid = private constant i8 0 ; Invalid, possibly uninitialised token
; @TokenTypeKeyword = private constant i8 1 ; fn, goto, goif
; @TokenTypeVerb = private constant i8 2 ; Basically a function call, e.g. dup, eq, _print
; @TokenTypeLiteral = private constant i8 3 ; Literals are pushed onto the stack immediately, e.g. 1, 73.4, "string", :1
; @TokenTypeDeclaration = private constant i8 4 ; A function declaration, including the equals, e.g. main =

; @TokenSubtypeKeyword_Fn = private constant i8 0 ; fn
; @TokenSubtypeKeyword_Goto = private constant i8 1 ; goto
; @TokenSubtypeKeyword_Goif = private constant i8 2 ; goif

; @TokenSubtypeVerb_CompDef = private constant i8 0 ; Compiler defined function, e.g. _print
; @TokenSubtypeVerb_NativeDef = private constant i8 1 ; Tower defined function

; @TokenSubtypeLiteral_Str = private constant i8 0
; @TokenSubtypeLiteral_Bool = private constant i8 1
; @TokenSubtypeLiteral_Label = private constant i8 2
; @TokenSubtypeLiteral_I64 = private constant i8 3
; @TokenSubtypeLiteral_F64 = private constant i8 4

; @TokenSubtypeDeclaration_CompDef = private constant i8 0 ; Compiler defined function, e.g. _print
; @TokenSubtypeDeclaration_NativeDef = private constant i8 1 ; Tower defined function

define i32 @main(i32 %argc, i8** %argv) {
	; %str_ptr = getelementptr [14 x i8], [14 x i8]* @str, i32 0, i32 0
	; %1 = call i32 @puts(i8* %str_ptr)

	; %fmt_str_ptr = getelementptr [4 x i8], [4 x i8]* @fmt_str, i32 0, i32 0
	; %2 = call i32 (i8*, ...) @printf(i8* %fmt_str_ptr, i32 42)

	; TODO: Test the @is_ws function

	%filename_ptr = getelementptr inbounds i8*, i8** %argv, i64 1
	%filename = load i8*, i8** %filename_ptr
	%puts_ret_1 = call i32 @puts(i8* %filename)

	%file_contents_len_ptr = alloca i32, i32 4
	%file_contents = call i8* @read_file(i8* %filename, i32* %file_contents_len_ptr)
	%file_contents_len = load i32, i32* %file_contents_len_ptr

	; %puts_ret_2 = call i32 @puts(i8* %file_contents)

	; %lf_strp = getelementptr [2 x i8], [2 x i8]* @lf_str, i32 0, i32 0
	; %puts_ret_3 = call i32 @puts(i8* %lf_strp)

	; Allocate 4 bytes in which to store the tokens array length, and call the @tokenise function to return the array of tokens along with writing the length to the allocated array
	%tokens_len_ptr_i8 = call i8* @malloc(i32 4)
	%tokens_len_ptr = bitcast i8* %tokens_len_ptr_i8 to i32*
	%tokens = call i8* @tokenise(i8* %file_contents, i32 %file_contents_len, i32* %tokens_len_ptr)

	%tokens_len = load i32, i32* %tokens_len_ptr
	%dbg_tokens_len_strp = getelementptr [19 x i8], [19 x i8]* @dbg_tokens_len_str, i32 0, i32 0
	%printf_ret = call i32 (i8*, ...) @printf(i8* %dbg_tokens_len_strp, i32 %tokens_len)

	call void @free(i8* %tokens)
	call void @free(i8* %file_contents)

	ret i32 0
}

; Produce a stream of tokens... TODO Decide how to implement this
define i8* @tokenise(i8* %code, i32 %code_len, i32* %tokens_len_ptr) {
	; TODO: Implement a lexer
	; TODO: Remember about comments

	; TODO: Need to write some documentation about tower and it's syntax

	entry:
		; Read the size of a single token from the global variable @token_size, then multiply that by 16 and allocate the result of that number of bytes
		; on the heap to store tokens
		; May have to rework this, still haven't figured out how exactly I am gonna store the tokens
		%token_size = load i32, i32* @token_size
		%tokens_len = mul i32 %token_size, 16
		%tokens = call i8* @malloc(i32 %tokens_len)

		; Local variable for storing the index of the current last token in the %tokens array, as if it were a i8*
		%tokens_idx_ptr = alloca i32, i32 4
		store i32 0, i32* %tokens_idx_ptr

		; Local variable for storing the index of the about-to-be-examined character in the string %code
		%code_idx_ptr = alloca i32, i32 4
		store i32 0, i32* %code_idx_ptr

		br label %loop-init

	loop-init:
		; Load the current token index and current code character index from the respective pointers
		%tokens_idx = load i32, i32* %tokens_idx_ptr
		%code_idx = load i32, i32* %code_idx_ptr

		; Check whether we are at the end of the code by comparing: %code_idx >= %code_len
		%at_end_comp = icmp uge i32 %code_idx, %code_len

		; If we are at the end of the code, then exit, or go to %loop-body
		br i1 %at_end_comp, label %exit, label %loop-body

	loop-body:
		; Get a pointer to the current code character and dereference it to get it's value
		%code_char_ptr = getelementptr i8, i8* %code, i32 %code_idx
		; %code_char = load i8, i8* %code_char_ptr ; idk if this is needed

		; Check for the presence of the "fn" keyword
		%keyword_fn_strp = getelementptr [3 x i8], [3 x i8]* @keyword_fn_str, i32 0, i32 0
		%fn_comp = call i1 @str_eq(i8* %code_char_ptr, i32 0, i32 2, i8* %keyword_fn_strp, i32 0, i32 2)

		; If "fn" is found at the current code index, then go to the %kwrd-fn block to handle it
		br i1 %fn_comp, label %kwrd-fn, label %no-matches

	kwrd-fn:
		; TODO: Check that this is a valid instance of a keyword - Check that it is surrounded by whitespace (need a function like @char_is_any) and
		; that if it is not a compiler function (starts with an _ (or maybe some other identifier idk)) it has an = and a body, also check that
		; the function has a valid name

		br label %kwrd-fn-check-ws

	kwrd-fn-check-ws:
		; Print "Keyword found at index: %i" (where obviously the %i is the index)
		%dbg_lex_kwrd_strp = getelementptr [28 x i8], [28 x i8]* @dbg_lex_kwrd_str, i32 0, i32 0
		%printf_ret = call i32 (i8*, ...) @printf(i8* %dbg_lex_kwrd_strp, i32 %code_idx)

		; Doesn't (yet) take into account that there may be compiler functions
		; %equals_sign_char = load i8, i8* @equals_sign
		; %equals_sign_idx = call i32 @str_find(i8* %code, i8 %equals_sign_char, i32 %code_idx, i32 %code_len)
		; call void @print_span(i8* %code_char_ptr, i32 0, i32 %equals_sign_idx)

		; Add to the current code index by 3 to skip the fn keyword
		%kwrd_new_code_idx = add i32 %code_idx, 3
		store i32 %kwrd_new_code_idx, i32* %code_idx_ptr

		; Go back to the start of the loop
		br label %loop-init

	no-matches:
		; Add to the current code index by 1 to move on
		%default_new_code_idx = add i32 %code_idx, 1
		store i32 %default_new_code_idx, i32* %code_idx_ptr

		; Go back to the start of the loop
		br label %loop-init

	exit:
		; Write the length of the %tokens array to the %tokens_len_ptr, and then return the %tokens array
		store i32 %tokens_len, i32* %tokens_len_ptr
		ret i8* %tokens
}

; Opens and reads the file at filename into a buffer of the file size on the heap
define i8* @read_file(i8* %filename, i32* %flen_ptr) {
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

	; Write the file length to the i32 value pointed to by the argument %flen_ptr
	store i32 %flen, i32* %flen_ptr

	; Return the file contents
	ret i8* %buf_ptr
}

define i1 @str_eq(i8* %str0, i32 %str0_start, i32 %str0_end, i8* %str1, i32 %str1_start, i32 %str1_end) {
	entry:
		; Check the ranges of the start to end for each string to compare match
		%str0_range = sub i32 %str0_end, %str0_start
		%str1_range = sub i32 %str1_end, %str1_start

		%range_comp = icmp eq i32 %str0_range, %str1_range

		; Go to %loop if %range_comp is true, %exit-uneq if false
		br i1 %range_comp, label %loop, label %exit-uneq

	loop:
		; i is 0 if we've just entered the loop, or the next_i if we're already in the loop
		%i = phi i32 [ 0, %entry ], [ %next_i, %continue ]

		; Compute indicies and then pointers to each character, and then dereference them to get the character
		%str0_idx = add i32 %str0_start, %i
		%str0_char_ptr = getelementptr i8, i8* %str0, i32 %str0_idx
		%char0 = load i8, i8* %str0_char_ptr

		%str1_idx = add i32 %str1_start, %i
		%str1_char_ptr = getelementptr i8, i8* %str1, i32 %str1_idx
		%char1 = load i8, i8* %str1_char_ptr

		; Compare the two characters
		%char_comp = icmp eq i8 %char0, %char1

		; If the characters are equal, jump to %continue - otherwise, jump to %exit-uneq and return with 0 (false)
		br i1 %char_comp, label %continue, label %exit-uneq

	continue:
		%next_i = add i32 %i, 1

		; Compare the current string index and the end of the string
		%at_end_comp = icmp eq i32 %str0_idx, %str0_end

		; If the current string index is equal to the index past the end of the string, then go to %exit-eq and return with 1 (true) otherwise jump back to %loop
		br i1 %at_end_comp, label %exit-eq, label %loop

	exit-uneq: ; Return false
		ret i1 0

	exit-eq: ; Return true
		ret i1 1
}

define i32 @str_find(i8* %str, i8 %seek_char, i32 %start, i32 %end) {
	entry:
		br label %loop

	loop:
		; If coming from entry, set %i to %start, if coming from %continue, set it to %next_i (i + 1)
		%i = phi i32 [ %start, %entry ], [ %next_i, %continue ]

		; Get a pointer to the current character in %str and dereference it to get the character value
		%curr_char_ptr = getelementptr i8, i8* %str, i32 %i
		%curr_char = load i8, i8* %curr_char_ptr

		; Compare the character with the sought character
		%char_comp = icmp eq i8 %curr_char, %seek_char

		; If the current character is equal to the one we're looking for, then jump to %exit, otherwise jump to %continue
		br i1 %char_comp, label %exit, label %continue

	continue:
		; i + 1
		%next_i = add i32 %i, 1

		; Check if we're at the end of the string
		%at_end_comp = icmp eq i32 %i, %end

		; If so then go to %exit, otherwise start back at %loop
		br i1 %at_end_comp, label %exit, label %loop

	exit:
		; If we came from %loop, then the return value will be %i since we only come from %loop when we've found a match
		; Otherwise, we came from continue, indicating we traversed the whole string without finding a match, so return -1
		%ret_i = phi i32 [ %i, %loop ], [ -1, %continue ]
		ret i32 %ret_i
}

define void @print_span(i8* %src_str, i32 %start, i32 %end) { ; Prints a section of a mutable string by inserting a null value at %end and calling printf
	; Get pointer to start of string
	%str_ptr = getelementptr i8, i8* %src_str, i32 %start

	; Insert a null character at the end of the substring, saving the previous character
	%str_end_ptr = getelementptr i8, i8* %src_str, i32 %end
	%char_end = load i8, i8* %str_end_ptr
	store i8 0, i8* %str_end_ptr

	; Print the substring
	%printf_ret = call i32 (i8*, ...) @printf(i8* %str_ptr)

	; Restore the end character to what it was before
	store i8 %char_end, i8* %str_end_ptr

	ret void
}

define i1 @is_ws(i8 %char) {
	; Get ws character string
	%ws_chars_strp = getelementptr [4 x i8], [4 x i8]* @ws_chars_str, i32 0, i32 0

	; Call @char_is_any to compare the given char to the whitespace chars
	%is_ws_comp = call i1 @char_is_any(i8 %char, i8* %ws_chars_strp, i32 4)

	ret i1 %is_ws_comp
}

define i1 @char_is_any(i8 %char, i8* %comp_chars, i32 %comp_chars_len) {
	entry:
		br label %loop

	loop:
		; If coming from entry, set %i to %start, if coming from %continue, set it to %next_i (i + 1)
		%i = phi i32 [ %start, %entry ], [ %next_i, %continue ]

		; Fetch the char at index %i in %comp_chars
		%curr_comp_char_ptr = getelementptr i8, i8* %comp_chars, i32 %i
		%curr_comp_char = load i8, i8* %curr_comp_char_ptr

		; Compare the input character with the one in the comp_chars array
		%char_eq_comp = icmp eq i8 %char, %curr_comp_char

		; If equal, then exit, if not equal, then continue
		br i1 %char_eq_comp, label %exit, label %continue

	continue:
		; i + 1
		%next_i = add i32 %i, 1

		; Check if we're at the end of %comp_chars
		%at_end_comp = icmp eq i32 %i, %comp_chars_len

		; If so then go to %exit, otherwise start back at %loop
		br i1 %at_end_comp, label %exit, label %loop

	exit:
		; If we came from %loop, then the return value will be true (1) since we only come from %loop when we've found a match
		; Otherwise, we came from continue, indicating we traversed the whole comparison character array without finding a match, so return false (0)
		%ret_comp = phi i32 [ 1, %loop ], [ 0, %continue ]
		ret i1 %ret_comp
}

; Forward declarations of used c library functions
declare i32 @puts(i8*) ; string -> error code
declare i32 @printf(i8*, ...) ; format string, ...arguments -> error code
declare i8* @fopen(i8*, i8*) ; filename, mode -> FILE
declare i32 @fseek(i8*, i32, i32) ; FILE, offset, origin (0 = start of file, 1 = current position in file, 2 = end of file) -> error code
declare i32 @ftell(i8*) ; FILE -> pos of FILE pointer
declare i8* @malloc(i32) ; size -> ptr to allocated memory
declare void @rewind(i8*) ; FILE
declare i64 @fread(i8*, i64, i64, i8*) ; write buffer, element size, num elements, FILE -> num read bytes
declare i32 @fclose(i8*) ; FILE -> error code
declare void @free(i8*) ; ptr to allocated memory
declare i8* @realloc(i8*) ; ptr to allocated memory -> ptr to newly resized allocated memory