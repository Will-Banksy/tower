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
; declare i8* @realloc(i8*) ; ptr to allocated memory -> ptr to newly resized allocated memory

; Forward declarations of types from other modules
%Token = type opaque

; Forward declarations of functions from other modules
; lexer.ll
declare %Token* @tokenise(i8* %code, i32 %code_len, i32* %tokens_len_ptr)
declare void @dbg_tokens(%Token* %tokens, i32 %tokens_len)

; Constants
@read_mode_cstr = private constant [2 x i8] c"r\00"
@dbg_tokens_len_cstr = private constant [19 x i8] c"Tokens length: %i\0A\00"

; TODO: Also decide how conditionals are going to work in tower - we want it simplistic as possible ideally but we might need some more keywords
; Also do I allow overloading or not?
; A way to handle conditionals *and* loops - goto and goif. Have a syntax for declaring labels, and have labels as a value, and then implement conditional goto label (scoped?)
;     e.g. 1:1 "loop" print goto
;          1 1 eq :1 goif "1 != 1" print 1:
; I might scrap the what factor calls "stack effect declarations" cause I'd need a good way to represent it when it might be conditional etc.
;     But maybe the conditional stuff can just be like "idk" and those functions aren't checked or maybe you can say "idk" to inputs/outputs

define i32 @main(i32 %argc, i8** %argv) {
	; Read the filename from the first argument of %argv
	%filename_ptr = getelementptr inbounds i8*, i8** %argv, i32 1
	%filename = load i8*, i8** %filename_ptr
	%puts_ret_1 = call i32 @puts(i8* %filename)

	; Allocate space on the stack to store the file length, then call @read_file with the %filename, and a pointer to the allocated stack
	; space that the function will write the file length to, which returns a pointer to the read file contents
	; Then dereference the %file_contents_len_ptr to get the length of the read file contents
	%file_contents_len_ptr = alloca i32, i32 1
	%file_contents = call i8* @read_file(i8* %filename, i32* %file_contents_len_ptr)
	%file_contents_len = load i32, i32* %file_contents_len_ptr

	; %puts_ret_2 = call i32 @puts(i8* %file_contents)

	; Allocate 4 bytes in which to store the tokens array length, and call the @tokenise function to return the array of tokens
	; along with writing the length to the allocated space
	; Dereference the %tokens_len_ptr to get the length of the tokens (how many %Tokens there are)
	%tokens_len_ptr = alloca i32, i32 1
	%tokens = call %Token* @tokenise(i8* %file_contents, i32 %file_contents_len, i32* %tokens_len_ptr)
	%tokens_len = load i32, i32* %tokens_len_ptr

	; Print out all the found tokens
	call void @dbg_tokens(%Token* %tokens, i32 %tokens_len)

	; Print the %tokens_len
	%dbg_tokens_len_strp = getelementptr [19 x i8], [19 x i8]* @dbg_tokens_len_cstr, i32 0, i32 0
	%printf_ret = call i32 (i8*, ...) @printf(i8* %dbg_tokens_len_strp, i32 %tokens_len)

	; Free memory allocated with @malloc (casting to a voidptr when necessary)
	%tokens_malloc = bitcast %Token* %tokens to i8*
	call void @free(i8* %tokens_malloc)
	call void @free(i8* %file_contents)

	ret i32 0
}

; Opens and reads the file at filename into a buffer of the file size on the heap
define i8* @read_file(i8* %filename, i32* %flen_ptr) {
	; Open the file
	%read_mode_strp = getelementptr [2 x i8], [2 x i8]* @read_mode_cstr, i32 0, i32 0
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