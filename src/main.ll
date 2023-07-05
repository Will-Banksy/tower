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

; Forward declarations of functions from other modules

; TODO: Consider declaring functions from this module here, as an easy way to see what functions are present

; TODO: Also decide how conditionals are going to work in tower - we want it simplistic as possible ideally but we might need some more keywords
; Also do I allow overloading or not?
; A way to handle conditionals *and* loops - goto and goif. Have a syntax for declaring labels, and have labels as a value, and then implement conditional goto label (scoped?)
;     e.g. 1:1 "loop" print goto
;          1 1 eq :1 goif "1 != 1" print 1:
; I might scrap the what factor calls "stack effect declarations" cause I'd need a good way to represent it when it might be conditional etc.
;     But maybe the conditional stuff can just be like "idk" and those functions aren't checked or maybe you can say "idk" to inputs/outputs

@token_size = private constant i32 16
@token_attr_type = private constant i32 0
@token_attr_subtype = private constant i32 1
@token_attr_data = private constant i32 2

%Token = type {
	i32, ; token type (e.g. keyword, verb, literal)
	i32, ; token subtype
	i32, ; data len
	i8* ; data
}

define %Token @Token.new(i32 %tok_type, i32 %tok_subtype, i32 %tok_data_len, i8* %tok_data) {
	%tok_ptr = alloca %Token

	; Write the values passed in as parameters to each field
	%type_ptr = bitcast %Token* %tok_ptr to i32*
	store i32 %tok_type, i32* %type_ptr
	%subtype_ptr = getelementptr inbounds %Token, %Token* %tok_ptr, i32 0, i32 1
	store i32 %tok_subtype, i32* %subtype_ptr
	%data_len_ptr = getelementptr inbounds %Token, %Token* %tok_ptr, i32 0, i32 2
	store i32 %tok_data_len, i32* %data_len_ptr
	%data_ptr = getelementptr inbounds %Token, %Token* %tok_ptr, i32 0, i32 3
	store i8* %tok_data, i8** %data_ptr

	; Then dereference and return the %Token
	%token = load %Token, %Token* %tok_ptr
	ret %Token %token
}

define i32 @Token.type(%Token* %tok) {
	%type_ptr = bitcast %Token* %tok to i32*
	%type = load i32, i32* %type_ptr

	ret i32 %type
}

define i32 @Token.subtype(%Token* %tok) {
	%subtype_ptr = getelementptr inbounds %Token, %Token* %tok, i32 0, i32 1
	%subtype = load i32, i32* %subtype_ptr

	ret i32 %subtype
}

define i32 @Token.data_len(%Token* %tok) {
	%data_len_ptr = getelementptr inbounds %Token, %Token* %tok, i32 0, i32 2
	%data_len = load i32, i32* %data_len_ptr

	ret i32 %data_len
}

define i8* @Token.data(%Token* %tok) {
	%data_ptr = getelementptr inbounds %Token, %Token* %tok, i32 0, i32 3
	%data = load i8*, i8** %data_ptr

	ret i8* %data
}

; Constant values for token type
@token_type_none = private constant i32 0 ; none: invalid token
@token_type_keyword = private constant i32 1 ; fn, goto, goif
@token_type_literal = private constant i32 2 ; 1, 9.2, "hello", :label
@token_type_ident = private constant i32 3 ; dup, _print
@token_type_label = private constant i32 4 ; label:

; Constant values for token subtype of type keyword
@token_subtype_keyword_fn = private constant i32 0
@token_subtype_keyword_fndef = private constant i32 1 ; =
@token_subtype_keyword_goto = private constant i32 2
@token_subtype_keyword_goif = private constant i32 3

; Type fndef has no subtypes

; Constant values for token subtype of type literal
@token_subtype_literal_i64 = private constant i32 0
@token_subtype_literal_u64 = private constant i32 1
@token_subtype_literal_f64 = private constant i32 2
@token_subtype_literal_bool = private constant i32 3
@token_subtype_literal_str = private constant i32 4
@token_subtype_literal_label = private constant i32 5

; Type ident has no subtypes
; Type label has no subtypes

@read_mode_cstr = private constant [2 x i8] c"r\00"
@fmt_int_cstr = private constant [4 x i8] c"%i\0A\00"
@fmt_uint_cstr = private constant [4 x i8] c"%u\0A\00"
@lf_cstr = private constant [2 x i8] c"\0A\00" ; "\n\0"
@ws_chars_str = private constant [4 x i8] c"\0A\0D\09\20" ; "\n\r\t "
@equals_sign = private constant i8 61 ; '='

@dbg_lex_found_token_cstr = private constant [31 x i8] c"Token found at code index: %i\0A\00"
@dbg_tokens_len_cstr = private constant [19 x i8] c"Tokens length: %i\0A\00"
@dbg_token_cstr = private constant [43 x i8] c"Token(type = %u, subtype = %u, data = %u)\0A\00"

@keyword_fn_str = private constant [2 x i8] c"fn"
@keyword_fndef_str = private constant [1 x i8] c"="

@literal_bool_false_str = private constant [5 x i8] c"false"
@literal_bool_true_str = private constant [4 x i8] c"true"

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

; Produce a stream of tokens from tower source code
define %Token* @tokenise(i8* %code, i32 %code_len, i32* %tokens_len_ptr) {
	; TODO: Implement a lexer
	; TODO: Remember about comments
	; TODO: Need to write some documentation about tower and it's syntax

	entry:
		; Read the size of a single token from the global variable @token_size, then multiply that by 16 and allocate the result of that number of bytes
		; on the heap to store tokens
		; May have to rework this, still haven't figured out how exactly I am gonna store the tokens
		%token_size = load i32, i32* @token_size
		%tokens_len = add i32 16, 0
		%tokens_size = mul i32 %token_size, %tokens_len
		%tokens_malloc = call i8* @malloc(i32 %tokens_size)
		%tokens = bitcast i8* %tokens_malloc to %Token*

		; Local variable for storing the index of the current last token in the %tokens array, as if it were a i8*
		%tokens_idx_ptr = alloca i32, i32 4
		store i32 0, i32* %tokens_idx_ptr

		; Local variable for storing the index of the about-to-be-examined character in the string %code
		%code_idx_ptr = alloca i32, i32 4
		store i32 0, i32* %code_idx_ptr

		; Dereference global constants
		%token_type_keyword = load i32, i32* @token_type_keyword
		%token_subtype_keyword_fn = load i32, i32* @token_subtype_keyword_fn
		%token_subtype_keyword_fndef = load i32, i32* @token_subtype_keyword_fndef
		%token_type_literal = load i32, i32* @token_type_literal
		%token_subtype_literal_label = load i32, i32* @token_subtype_literal_label

		%dbg_lex_found_token_cstr = getelementptr [31 x i8], [31 x i8]* @dbg_lex_found_token_cstr, i32 0, i32 0

		; Enter the main lexer loop
		br label %loop-init

	loop-init:
		; Load whether the previous character was ws or start of file with a phi
		%prev_is_ws_comp = phi i1 [ 1, %entry ], [ %atend_curr_is_ws_comp, %continue ]

		; Load the current token index and current code character index from the respective pointers
		%tokens_idx = load i32, i32* %tokens_idx_ptr
		%code_idx = load i32, i32* %code_idx_ptr

		; Check whether we are at the end of the code by comparing: %code_idx >= %code_len
		%at_end_comp = icmp uge i32 %code_idx, %code_len

		; If we are at the end of the code, then exit, or go to %loop-body
		br i1 %at_end_comp, label %exit, label %loop-body

	loop-body:
		; Get a pointer to the current code character and dereference it to get its value
		%code_char_ptr = getelementptr i8, i8* %code, i32 %code_idx
		%code_char = load i8, i8* %code_char_ptr

		br label %loop-body-skip-ws

	loop-body-skip-ws: ; If the current code character is whitespace, just continue to the next character, otherwise go to checking for comments
		%code_char_is_ws_comp = call i1 @is_ws(i8 %code_char)

		br i1 %code_char_is_ws_comp, label %continue-inc1, label %loop-body-comments

	loop-body-comments: ; Check for comments ('#' until newline) and skip them
		%comment_comp = icmp eq i8 %code_char, 35 ; '#'

		br i1 %comment_comp, label %loop-body-comments-skip, label %loop-body-kwrds

	loop-body-comments-skip: ; Invoked if the current char is a comment, calculates the amount of characters to move to the next newline '\n'
		%code_last_idx = sub i32 %code_len, 1
		%comment_end = call i32 @str_find(i8* %code, i8 10, i32 %code_idx, i32 %code_last_idx)
		%comment_skip = sub i32 %comment_end, %code_idx

		br label %continue

	loop-body-kwrds:
		br label %loop-body-kwrd-fn

	loop-body-kwrd-fn: ; Check for the "fn" keyword, and if present add token
		%kwrd_fn_str = getelementptr [2 x i8], [2 x i8]* @keyword_fn_str, i32 0, i32 0
		%kwrd_fn_len = add i32 2, 0
		%kwrd_fn_comp = call i1 @check_keyword(i8* %code, i32 %code_len, i32 %code_idx, i8* %kwrd_fn_str, i32 %kwrd_fn_len)

		br i1 %kwrd_fn_comp, label %add-token, label %loop-body-kwrd-fndef

	loop-body-kwrd-fndef: ; Check for the "=" keyword, and if present add token
		%kwrd_fndef_str = getelementptr [1 x i8], [1 x i8]* @keyword_fndef_str, i32 0, i32 0
		%kwrd_fndef_len = add i32 1, 0
		%kwrd_fndef_comp = call i1 @check_keyword(i8* %code, i32 %code_len, i32 %code_idx, i8* %kwrd_fndef_str, i32 %kwrd_fndef_len)

		br i1 %kwrd_fndef_comp, label %add-token, label %loop-body-literals

	loop-body-literals: ; Check for literals using the @check_literal function
		%literal_size_ptr = alloca i32
		%literal_subtype = call i32 @check_literal(i8* %code, i32 %code_len, i32 %code_idx, i32* %literal_size_ptr)
		%literal_size = load i32, i32* %literal_size_ptr

		; @token_subtype_literal_i64 = private constant i32 0
		; @token_subtype_literal_u64 = private constant i32 1
		; @token_subtype_literal_f64 = private constant i32 2
		; @token_subtype_literal_bool = private constant i32 3
		; @token_subtype_literal_str = private constant i32 4
		; @token_subtype_literal_label = private constant i32 5

		; Switch requires constants/immediates
		; TODO: Finish
		switch i32 %literal_subtype, label %loop-body-literals-none [ i32 5, label %loop-body-literals-labels ]

	loop-body-literals-labels: ; TODO
		br label %add-token

	loop-body-literals-none:
		br label %continue-inc1

	loop-body-labels: ; TODO
		br label %continue-inc1

	loop-body-idents: ; TODO
		br label %continue-inc1

	add-token: ; NOTE: Could this be turned into a function?
		; Define values based on which token is being added
		%token_type = phi i32 [ %token_type_keyword, %loop-body-kwrd-fn ],
		                      [ %token_type_keyword, %loop-body-kwrd-fndef ],
		                      [ %token_type_literal, %loop-body-literals-labels ]

		%token_subtype = phi i32 [ %token_subtype_keyword_fn, %loop-body-kwrd-fn ],
		                         [ %token_subtype_keyword_fndef, %loop-body-kwrd-fndef ],
								 [ %token_subtype_literal_label, %loop-body-literals-labels ]

		%token_data_len = phi i32 [ 0, %loop-body-kwrd-fn ],
		                          [ 0, %loop-body-kwrd-fndef ],
								  [ 0, %loop-body-literals-labels ]

		%token_data = phi i8* [ null, %loop-body-kwrd-fn ],
		                      [ null, %loop-body-kwrd-fndef ],
							  [ null, %loop-body-literals-labels ]

		%token_skip = phi i32 [ 2, %loop-body-kwrd-fn ],
		                      [ 1, %loop-body-kwrd-fndef ],
							  [ %literal_size, %loop-body-literals-labels ]

		; Print "Keyword found at index: %i" (where obviously the %i is the index)
		; TODO: Change this. It is not only finding keywords that will execute this line
		%printf_ret = call i32 (i8*, ...) @printf(i8* %dbg_lex_found_token_cstr, i32 %code_idx)

		; Create and add a token to the tokens array
		%new_fn_token = call %Token @Token.new(i32 %token_type, i32 %token_subtype, i32 %token_data_len, i8* %token_data)
		%tokens_ptr = getelementptr %Token, %Token* %tokens, i32 %tokens_idx
		store %Token %new_fn_token, %Token* %tokens_ptr
		%postfn_tokens_idx = add i32 %tokens_idx, 1
		store i32 %postfn_tokens_idx, i32* %tokens_idx_ptr

		br label %continue

	continue-inc1:
		; Go to the %continue label
		br label %continue

	continue:
		; Define an increment amount for each label we're coming from, which basically means define an increment for each found thing otherwise 1
		%inc_amt = phi i32 [ 1, %continue-inc1 ], [ %token_skip, %add-token ], [ %comment_skip, %loop-body-comments-skip ]

		; Check the character at the current code ptr before increasing it
		%atend_curr_is_ws_comp = call i1 @is_ws(i8 %code_char)

		; Add to the current code index by whatever to move on
		%default_new_code_idx = add i32 %code_idx, %inc_amt
		store i32 %default_new_code_idx, i32* %code_idx_ptr

		; Go back to the start of the loop
		br label %loop-init

	exit:
		; Write the length of the %tokens array to the %tokens_len_ptr, and then return the %tokens array
		store i32 %tokens_idx, i32* %tokens_len_ptr
		ret %Token* %tokens
}

; Checks that starting at %code_idx, a valid literal is present. Returns -1 if no literal was found,
; or the number corresponding to the appropriate token subtype otherwise
define i32 @check_literal(i8* %code, i32 %code_len, i32 %code_idx, i32* %literal_size) {
	; Literals: i64, u64, f64, str, bool, str, label

	entry:
		br label %check-bool-literals

	check-bool-literals:
		br label %check-bool-literals-true

	check-bool-literals-true: ; Check for "true"
		%literal_bool_true_str = getelementptr [4 x i8], [4 x i8]* @literal_bool_true_str, i32 0, i32 0
		%literal_bool_true_len = add i32 4, 0
		%literal_bool_true_comp = call i1 @check_keyword(i8* %code, i32 %code_len, i32 %code_idx, i8* %literal_bool_true_str, i32 %literal_bool_true_len)

		br i1 %literal_bool_true_comp, label %return-literal-bool, label %check-bool-literals-false

	check-bool-literals-false: ; Check for "false"
		%literal_bool_false_str = getelementptr [5 x i8], [5 x i8]* @literal_bool_false_str, i32 0, i32 0
		%literal_bool_false_len = add i32 5, 0
		%literal_bool_false_comp = call i1 @check_keyword(i8* %code, i32 %code_len, i32 %code_idx, i8* %literal_bool_false_str, i32 %literal_bool_false_len)

		br i1 %literal_bool_false_comp, label %return-literal-bool, label %check-lab-literals

	check-lab-literals: ; Check if the current character is ':'
		%code_char_ptr = getelementptr i8, i8* %code, i32 %code_idx
		%code_char = load i8, i8* %code_char_ptr

		%code_char_colon_comp = icmp eq i8 %code_char, 58 ; ':'

		br i1 %code_char_colon_comp, label %check-lab-literals-name, label %return-none

	check-lab-literals-name: ; Get the index of the end of the label name
		%next_ws = call i32 @str_find_ws_or_end(i8* %code, i32 %code_idx, i32 %code_len)

		%code_next_idx = add i32 %code_idx, 1
		%lab_lit_len_zero_comp = icmp eq i32 %next_ws, %code_next_idx

		br i1 %lab_lit_len_zero_comp, label %return-none, label %check-lab-literals-name-is-ident

	check-lab-literals-name-is-ident:
		%code_last_before_ws_idx = sub i32 %next_ws, 1
		%label_ident_name_comp = call i1 @is_ident_str(i8* %code_char_ptr, i32 %code_next_idx, i32 %code_last_before_ws_idx, i1 1)

		br i1 %label_ident_name_comp, label %return-literal-lab, label %check-num-literals

	check-num-literals: ; TODO
		br label %return-none

	return-literal-bool:
		%bool_lit_size = phi i32 [ 4, %check-bool-literals-true ], [ 5, %check-bool-literals-false ]
		store i32 %bool_lit_size, i32* %literal_size

		%token_subtype_literal_bool = load i32, i32* @token_subtype_literal_bool
		ret i32 %token_subtype_literal_bool

	return-literal-lab:
		%lab_lit_size = sub i32 %next_ws, %code_idx ; TODO: Is this the correct calculation?
		store i32 %lab_lit_size, i32* %literal_size

		%token_subtype_literal_label = load i32, i32* @token_subtype_literal_label
		ret i32 %token_subtype_literal_label

	return-none:
		store i32 0, i32* %literal_size

		ret i32 -1
}



define i1 @is_ident_str(i8* %str, i32 %start_idx, i32 %end_idx, i1 %allow_num_at_start_comp) {
	entry:
		br label %loop

	loop:
		%i = phi i32 [ %start_idx, %entry ], [ %next_i, %loop-checkend ]
		%str_start_comp = phi i1 [ 1, %entry ], [ 0, %loop-checkend ]

		%next_i = add i32 %i, 1

		%not_num_allow_comp = xor i1 %allow_num_at_start_comp, 1 ; NOT is implemented with xor'ing an i1 with 1
		%num_allowed_comp = and i1 %str_start_comp, %not_num_allow_comp

		%str_char_ptr = getelementptr i8, i8* %str, i32 %i
		%str_char = load i8, i8* %str_char_ptr

		%ident_char_comp = call i1 @is_ident_char(i8 %str_char, i1 %num_allowed_comp)

		br label %loop-checkend

	loop-checkend:
		%at_end_comp = icmp eq i32 %i, %end_idx

		br i1 %at_end_comp, label %return-true, label %loop

	return-false:
		ret i1 0

	return-true:
		ret i1 1
}

define i1 @is_ident_char(i8 %char, i1 %allow_num) {
	; (%char >= 65 && %char <= 90) || (%char >= 97 && %char <= 122)
	%alpha_uc_lowerbound_comp = icmp uge i8 %char, 65 ; 'A'
	%alpha_uc_upperbound_comp = icmp ule i8 %char, 90 ; 'Z'
	%alpha_uc_comp = and i1 %alpha_uc_lowerbound_comp, %alpha_uc_upperbound_comp
	%alpha_lc_lowerbound_comp = icmp uge i8 %char, 97 ; 'a'
	%alpha_lc_upperbound_comp = icmp ule i8 %char, 122 ; 'z'
	%alpha_lc_comp = and i1 %alpha_lc_lowerbound_comp, %alpha_lc_upperbound_comp
	%alpha_comp = or i1 %alpha_uc_comp, %alpha_lc_comp

	; (%char >= 48 && %char <= 57) && %allow_num
	%num_lowerbound_comp = icmp uge i8 %char, 48 ; '0'
	%num_upperbound_comp = icmp ule i8 %char, 57 ; '9'
	%num_comp = and i1 %num_lowerbound_comp, %num_upperbound_comp
	%valid_num_comp = and i1 %num_comp, %allow_num

	%valid_so_far_comp = or i1 %alpha_comp, %valid_num_comp

	%underscore_comp = icmp eq i8 %char, 95 ; '_'

	%valid_ident_char_comp = or i1 %valid_so_far_comp, %underscore_comp

	ret i1 %valid_ident_char_comp
}

; Checks that starting at %code_idx, a valid keyword is present. Returns 1 if a valid keyword is found, 0 otherwise
define i1 @check_keyword(i8* %code, i32 %code_len, i32 %code_idx, i8* %keyword, i32 %keyword_len) {
	entry:
		%code_char_ptr = getelementptr i8, i8* %code, i32 %code_idx

		br label %check-at-start

	check-at-start: ; Check whether we are at the start of the string
		%at_start_comp = icmp eq i32 %code_idx, 0

		br i1 %at_start_comp, label %check-avail-space, label %check-ws-before

	check-ws-before: ; Check whether the character at %code_idx - 1 is a whitespace character
		%before_idx = sub i32 %code_idx, 1
		%before_ptr = getelementptr i8, i8* %code, i32 %before_idx
		%before_char = load i8, i8* %before_ptr
		%ws_before_comp = call i1 @is_ws(i8 %before_char)

		br i1 %ws_before_comp, label %check-avail-space, label %exit-false

	check-avail-space: ; Check whether %code_idx + %keyword_len is less than %code_len
		%after_idx = add i32 %code_idx, %keyword_len

		%avail_space_comp = icmp ult i32 %after_idx, %code_len

		br i1 %avail_space_comp, label %check-ws-after, label %check-at-end

	check-at-end: ; Check whether %code_idx + %keyword_len is equal to %code_len
		%at_end_comp = icmp eq i32 %after_idx, %code_len

		br i1 %at_end_comp, label %check-streq, label %exit-false

	check-ws-after: ; Check that there is whitespace at %code_idx + %keyword_len
		%after_ptr = getelementptr i8, i8* %code, i32 %after_idx
		%after_char = load i8, i8* %after_ptr
		%ws_after_comp = call i1 @is_ws(i8 %after_char)

		br i1 %ws_after_comp, label %check-streq, label %exit-false

	check-streq: ; Check that the string starting at %code_idx and advancing %keyword_len places is equal to %keyword
		%keyword_stop = sub i32 %keyword_len, 1
		%streq_comp = call i1 @str_eq(i8* %code_char_ptr, i32 0, i32 %keyword_stop, i8* %keyword, i32 0, i32 %keyword_stop)

		br i1 %streq_comp, label %exit-true, label %exit-false

	exit-false:
		ret i1 0

	exit-true:
		ret i1 1
}

define void @dbg_tokens(%Token* %tokens, i32 %tokens_len) {
	entry:
		; May as well just define this here. Avoids repeating the same calculation
		%dbg_token_cstr = getelementptr [43 x i8], [43 x i8]* @dbg_token_cstr, i32 0, i32 0

		br label %loop-init

	loop-init:
		; %i and %next_i - Just the variables tracking the current token index, and the next one
		%i = phi i32 [ 0, %entry ], [ %next_i, %loop-body-2 ]
		%next_i = add i32 %i, 1

		; If at end of %tokens array, then exit
		%at_end_comp = icmp eq i32 %i, %tokens_len
		br i1 %at_end_comp, label %exit, label %loop-body

	loop-body:
		; Retrieve a reference to the current %Token, and then use the accessor functions to retrieve its fields
		%tok_ptr = getelementptr %Token, %Token* %tokens, i32 %i
		%tok_type = call i32 @Token.type(%Token* %tok_ptr)
		%tok_subtype = call i32 @Token.subtype(%Token* %tok_ptr)
		%tok_data = call i8* @Token.data(%Token* %tok_ptr)

		; Load the global constant @token_type_none and compare the current token type against it
		%token_type_none = load i32, i32* @token_type_none
		%none_comp = icmp eq i32 %tok_type, %token_type_none

		; If the token type is equal to @token_type_none, then exit
		; TODO: Evaluate this
		br i1 %none_comp, label %exit, label %loop-body-2

	loop-body-2:
		; Cast the token data pointer to a number for printing
		%tok_data_num = ptrtoint i8* %tok_data to i32

		; Print out the retrieved token information
		%printf_res = call i32 (i8*, ...) @printf(i8* %dbg_token_cstr, i32 %tok_type, i32 %tok_subtype, i32 %tok_data_num)

		br label %loop-init

	exit:
		ret void
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

; Finds and returns the next instance of %seek_char starting at %start and ending at %end (inclusive) or -1 if no match
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

; Finds and returns the next instance of any whitespace starting at %start_idx and ending at %end_idx (exclusive) or %end_idx if no match
; NOTE: If necessary, rename this to @str_find_ws, make it return -1 if no match, then make another fn with the _or_end and call this fn and transform -1 to %end_idx
define i32 @str_find_ws_or_end(i8* %str, i32 %start_idx, i32 %end_idx) {
	entry:
		br label %loop

	loop:
		; If coming from entry, set %i to %start, if coming from %continue, set it to %next_i (i + 1)
		%i = phi i32 [ %start_idx, %entry ], [ %next_i, %continue ]

		; Get a pointer to the current character in %str and dereference it to get the character value
		%curr_char_ptr = getelementptr i8, i8* %str, i32 %i
		%curr_char = load i8, i8* %curr_char_ptr

		; Compare the character with the sought character
		%char_comp = call i1 @is_ws(i8 %curr_char)

		; If the current character is whitespace, then jump to %exit, otherwise jump to %continue
		br i1 %char_comp, label %exit, label %continue

	continue:
		; i + 1
		%next_i = add i32 %i, 1

		; Check if we're at the end of the string
		%at_end_comp = icmp eq i32 %i, %end_idx

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
		%i = phi i32 [ 0, %entry ], [ %next_i, %continue ]

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
		%ret_comp = phi i1 [ 1, %loop ], [ 0, %continue ]
		ret i1 %ret_comp
}
