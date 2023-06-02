; @str = private constant [14 x i8] c"Hello, world!\00"
; @fmt_str = private constant [4 x i8] c"%i\0A\00"

@read_mode_str = private constant [2 x i8] c"r\00"
@fmt_i32_str = private constant [4 x i8] c"%d\0A\00"

; @TokenTypeKeyword ...TODO

%LexerToken = type {
	i32, ; Token type
	i32 ; Token subtype
}

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

	ret i32 0
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

; Produce a stream of tokens... TODO Decide how to implement this
define void @tokenise(i8*) {
	; TODO: Implement a lexer
	ret void
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