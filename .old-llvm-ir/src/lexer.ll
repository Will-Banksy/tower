; Forward declarations of used c library functions
declare i8* @malloc(i32) ; size -> ptr to allocated memory
declare i32 @printf(i8*, ...) ; format string, ...arguments -> error code

; Forward declarations of functions from other modules
; str_utils.ll
declare i1 @str_eq(i8* %str0, i32 %str0_start, i32 %str0_end, i8* %str1, i32 %str1_start, i32 %str1_end)
declare i32 @str_find(i8* %str, i8 %seek_char, i32 %start, i32 %end)
declare i32 @str_find_ws_or_end(i8* %str, i32 %start_idx, i32 %end_idx)
declare i1 @is_ws(i8 %char)
declare i32 @rcount_continuous(i8* %str, i8 %counting_char, i32 %lower_bound, i32 %upper_bound)
declare i1 @is_num(i8 %char)
declare double @parse_f64(i8* %str, i32 %str_len)
declare i64 @parse_u64(i8* %str, i32 %str_len)
declare i64 @parse_i64(i8* %str, i32 %str_len)

@fmt_int_cstr = private constant [4 x i8] c"%i\0A\00"
@fmt_uint_cstr = private constant [4 x i8] c"%u\0A\00"
@lf_cstr = private constant [2 x i8] c"\0A\00" ; "\n\0"
@equals_sign = private constant i8 61 ; '='

@dbg_lex_found_token_cstr = private constant [31 x i8] c"Token found at code index: %i\0A\00"
@dbg_token_cstr = private constant [43 x i8] c"Token(type = %u, subtype = %u, data = %u)\0A\00"
@dbg_checkpoint_cstr = private constant [20 x i8] c"Hit checkpoint %i!\0A\00"

@keyword_fn_str = private constant [2 x i8] c"fn"
@keyword_fndef_str = private constant [1 x i8] c"="

@literal_bool_false_str = private constant [5 x i8] c"false"
@literal_bool_true_str = private constant [4 x i8] c"true"

; Constants for tokens
@token_size = private constant i32 20
@token_attr_type = private constant i32 0
@token_attr_subtype = private constant i32 1
@token_attr_data = private constant i32 2

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
		%token_subtype_literal_str = load i32, i32* @token_subtype_literal_str
		%token_subtype_literal_bool = load i32, i32* @token_subtype_literal_bool
		%token_subtype_literal_f64 = load i32, i32* @token_subtype_literal_f64
		%token_subtype_literal_u64 = load i32, i32* @token_subtype_literal_u64
		%token_subtype_literal_i64 = load i32, i32* @token_subtype_literal_i64

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
		%literal_code_size_ptr = alloca i32
		%literal_subtype = call i32 @parse_literal(i8* %code, i32 %code_len, i32 %code_idx, i32* %literal_code_size_ptr, i8** null)
		%literal_code_size = load i32, i32* %literal_code_size_ptr

		; @token_subtype_literal_i64 = private constant i32 0
		; @token_subtype_literal_u64 = private constant i32 1
		; @token_subtype_literal_f64 = private constant i32 2
		; @token_subtype_literal_bool = private constant i32 3
		; @token_subtype_literal_str = private constant i32 4
		; @token_subtype_literal_label = private constant i32 5

		; Switch requires constants/immediates
		; TODO: Create tokens for each literal type.
		; TODO: Think about how I'm gonna handle token data
		switch i32 %literal_subtype, label %loop-body-literals-none [ i32 5, label %loop-body-literals-labels
		                                                              i32 4, label %loop-body-literals-strs
																	  i32 3, label %loop-body-literals-bools
																	  i32 2, label %loop-body-literals-f64
																	  i32 1, label %loop-body-literals-u64
																	  i32 0, label %loop-body-literals-i64 ]

	loop-body-literals-labels:
		br label %add-token

	loop-body-literals-strs:
		br label %add-token

	loop-body-literals-bools:
		br label %add-token

	loop-body-literals-f64:
		br label %add-token

	loop-body-literals-u64:
		br label %add-token

	loop-body-literals-i64:
		br label %add-token

	; TODO: The rest of the token types calling %add-token

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
		                      [ %token_type_literal, %loop-body-literals-labels ],
		                      [ %token_type_literal, %loop-body-literals-strs ],
		                      [ %token_type_literal, %loop-body-literals-bools ],
		                      [ %token_type_literal, %loop-body-literals-f64 ],
		                      [ %token_type_literal, %loop-body-literals-u64 ],
		                      [ %token_type_literal, %loop-body-literals-i64 ]

		%token_subtype = phi i32 [ %token_subtype_keyword_fn, %loop-body-kwrd-fn ],
		                         [ %token_subtype_keyword_fndef, %loop-body-kwrd-fndef ],
								 [ %token_subtype_literal_label, %loop-body-literals-labels ],
								 [ %token_subtype_literal_str, %loop-body-literals-strs ],
								 [ %token_subtype_literal_bool, %loop-body-literals-bools ],
								 [ %token_subtype_literal_f64, %loop-body-literals-f64 ],
								 [ %token_subtype_literal_u64, %loop-body-literals-u64 ],
								 [ %token_subtype_literal_i64, %loop-body-literals-i64 ]

		%token_data_len = phi i32 [ 0, %loop-body-kwrd-fn ],
		                          [ 0, %loop-body-kwrd-fndef ],
								  [ 0, %loop-body-literals-labels ],
								  [ 0, %loop-body-literals-strs ],
								  [ 0, %loop-body-literals-bools ],
								  [ 0, %loop-body-literals-f64 ],
								  [ 0, %loop-body-literals-u64 ],
								  [ 0, %loop-body-literals-i64 ]

		%token_data = phi i8* [ null, %loop-body-kwrd-fn ],
		                      [ null, %loop-body-kwrd-fndef ],
							  [ null, %loop-body-literals-labels ],
							  [ null, %loop-body-literals-strs ],
							  [ null, %loop-body-literals-bools ],
							  [ null, %loop-body-literals-f64 ],
							  [ null, %loop-body-literals-u64 ],
							  [ null, %loop-body-literals-i64 ]

		%token_skip = phi i32 [ 2, %loop-body-kwrd-fn ],
		                      [ 1, %loop-body-kwrd-fndef ],
							  [ %literal_code_size, %loop-body-literals-labels ],
							  [ %literal_code_size, %loop-body-literals-strs ],
							  [ %literal_code_size, %loop-body-literals-bools ],
							  [ %literal_code_size, %loop-body-literals-f64 ],
							  [ %literal_code_size, %loop-body-literals-u64 ],
							  [ %literal_code_size, %loop-body-literals-i64 ]

		; Print "Keyword found at index: %i" (where obviously the %i is the index)
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

; Parses a literal at %code_idx. Returns -1 if no literal was found,
; or the number corresponding to the appropriate token subtype otherwise.
; If literal is found, allocates space on heap and writes the literal data to it,
; writing the pointer value to %literal_data_ptr
define i32 @parse_literal(i8* %code, i32 %code_len, i32 %code_idx, i32* %literal_code_size, i8** %literal_data_ptr) {
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
		%code_next_idx = add i32 %code_idx, 1

		%code_char_colon_comp = icmp eq i8 %code_char, 58 ; ':'

		br i1 %code_char_colon_comp, label %check-lab-literals-name, label %check-num-literals ; TODO: Once done check-num-literals, go to that label instead

	check-lab-literals-name: ; Get the index of the end of the label name
		%next_ws = call i32 @str_find_ws_or_end(i8* %code, i32 %code_idx, i32 %code_len)

		%lab_lit_len_zero_comp = icmp eq i32 %next_ws, %code_next_idx

		br i1 %lab_lit_len_zero_comp, label %return-none, label %check-lab-literals-name-is-ident

	check-lab-literals-name-is-ident:
		%code_last_before_ws_idx = sub i32 %next_ws, 1
		%label_ident_name_comp = call i1 @is_ident_str(i8* %code_char_ptr, i32 %code_next_idx, i32 %code_last_before_ws_idx, i1 1)

		br i1 %label_ident_name_comp, label %return-literal-lab, label %return-none

	check-num-literals: ; TODO Implement numeric literal parsing (there's some c library functions that can do this but I need to ensure it follows my rules)
		call void @dbg_hit_checkpoint(i32 0)
		; Check if the first character is a number
		%is_char_num = call i1 @is_num(i8 %code_char)
		br i1 %is_char_num, label %check-num-literals-check-decimal, label %check-str-literals

	check-num-literals-check-decimal:
		call void @dbg_hit_checkpoint(i32 1)
		; Find the next ws or end, and then in that region try to find a '.'
		%num_end_idx = call i32 @str_find_ws_or_end(i8* %code, i32 %code_idx, i32 %code_len)
		%num_last_idx = sub i32 %num_end_idx, 1
		%num_strlen = sub i32 %num_end_idx, %code_idx ; Used later
		%num_decimal_pt_idx = call i32 @str_find(i8* %code, i8 46, i32 %code_idx, i32 %num_last_idx)
		%num_has_decimal_pt_comp = icmp ne i32 %num_decimal_pt_idx, -1
		br label %check-num-literals-check-suffix-f

	check-num-literals-check-suffix-f:
		%num_last_ptr = getelementptr i8, i8* %code, i32 %num_last_idx
		%num_last_char = load i8, i8* %num_last_ptr
		%num_explicitly_f = icmp eq i8 %num_last_char, 102 ; 'f'

		%num_explicitly_typed_not_comp = call i1 @is_num(i8 %num_last_char)
		%num_explicitly_typed_comp = xor i1 %num_explicitly_typed_not_comp, 1 ; NOT

		%num_is_f64_comp = or i1 %num_has_decimal_pt_comp, %num_explicitly_f
		br i1 %num_is_f64_comp, label %check-num-literals-f64, label %check-num-literals-check-suffix-u

	check-num-literals-check-suffix-u:
		call void @dbg_hit_checkpoint(i32 2)
		%num_explicitly_u = icmp eq i8 %num_last_char, 117 ; 'u'

		br i1 %num_explicitly_u, label %check-num-literals-u64, label %check-num-literals-i64

	check-num-literals-f64:
		%num_f64 = call double @parse_f64(i8* %code_char_ptr, i32 %num_strlen)
		%token_subtype_literal_f64 = load i32, i32* @token_subtype_literal_f64
		br label %return-literal-num

	check-num-literals-u64:
		%num_u64 = call i64 @parse_u64(i8* %code_char_ptr, i32 %num_strlen)
		%token_subtype_literal_u64 = load i32, i32* @token_subtype_literal_u64
		br label %return-literal-num

	check-num-literals-i64:
		%num_i64 = call i64 @parse_i64(i8* %code_char_ptr, i32 %num_strlen)
		%token_subtype_literal_i64 = load i32, i32* @token_subtype_literal_i64
		br label %return-literal-num

	check-str-literals:
		; If the current character is ", then search for an unescaped closing "
		%is_open_quote_comp = icmp eq i8 %code_char, 34 ; '"'
		br i1 %is_open_quote_comp, label %check-str-literals-find-closing, label %return-none

	check-str-literals-find-closing:
		%start_quote_find_idx = phi i32 [ %code_next_idx, %check-str-literals ], [ %after_maybe_close_quote_idx, %check-str-literals-find-closing ]

		%code_last_idx = sub i32 %code_len, 1
		%maybe_close_quote_idx = call i32 @str_find(i8* %code, i8 34, i32 %start_quote_find_idx, i32 %code_last_idx) ; BUG we don't handle if no found
		%before_maybe_close_quote_idx = sub i32 %maybe_close_quote_idx, 1
		%after_maybe_close_quote_idx = add i32 %maybe_close_quote_idx, 1

		; '\' = 92
		%num_backslashes_before_close = call i32 @rcount_continuous(i8* %code, i8 92, i32 0, i32 %before_maybe_close_quote_idx)
		%num_backslashes_mod2 = urem i32 %num_backslashes_before_close, 2
		%is_quote_escaped = icmp ne i32 %num_backslashes_mod2, 0

		br i1 %is_quote_escaped, label %check-str-literals-find-closing, label %return-literal-str

	return-literal-bool:
		%bool_lit_size = phi i32 [ 4, %check-bool-literals-true ], [ 5, %check-bool-literals-false ]
		; TODO: Allocate and write literal data
		store i32 %bool_lit_size, i32* %literal_code_size

		%token_subtype_literal_bool = load i32, i32* @token_subtype_literal_bool
		ret i32 %token_subtype_literal_bool

	return-literal-lab:
		%lab_lit_size = sub i32 %next_ws, %code_idx ; TODO: Is this the correct calculation?
		; TODO: Allocate and write literal data
		store i32 %lab_lit_size, i32* %literal_code_size

		%token_subtype_literal_label = load i32, i32* @token_subtype_literal_label
		ret i32 %token_subtype_literal_label

	return-literal-num:
		%num_lit_token_subtype = phi i32 [ %token_subtype_literal_f64, %check-num-literals-f64 ],
		                                 [ %token_subtype_literal_u64, %check-num-literals-u64 ],
		                                 [ %token_subtype_literal_i64, %check-num-literals-i64 ]
		%num_lit_size = add i32 %num_strlen, 0
		store i32 %num_lit_size, i32* %literal_code_size
		; TODO: Allocate and write literal data
		ret i32 %num_lit_token_subtype

	return-literal-str:
		%str_lit_size_minus_1 = sub i32 %maybe_close_quote_idx, %code_idx
		; TODO: Allocate and write literal data
		%str_lit_size = add i32 %str_lit_size_minus_1, 1
		store i32 %str_lit_size, i32* %literal_code_size

		%token_subtype_literal_str = load i32, i32* @token_subtype_literal_str
		ret i32 %token_subtype_literal_str

	return-none:
		store i32 0, i32* %literal_code_size

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

define void @dbg_hit_checkpoint(i32 %num) {
	%dbg_checkpoint_cstr = getelementptr [20 x i8], [20 x i8]* @dbg_checkpoint_cstr, i32 0, i32 0
	%printf_ret = call i32 (i8*, ...) @printf(i8* %dbg_checkpoint_cstr, i32 %num)
	ret void
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