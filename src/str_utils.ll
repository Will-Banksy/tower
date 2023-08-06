; Constants
@ws_chars_str = private constant [4 x i8] c"\0A\0D\09\20" ; "\n\r\t "

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

; Counts the number of occurrences of %counting_char backwards starting at %upper_bound and ending at %lower_bound (inclusive)
define i32 @rcount_continuous(i8* %str, i8 %counting_char, i32 %lower_bound, i32 %upper_bound) {
	entry:
		br label %loop

	loop:
		%i = phi i32 [ %upper_bound, %entry ], [ %next_i, %continue ]

		%str_char_ptr = getelementptr i8, i8* %str, i32 %i
		%str_char = load i8, i8* %str_char_ptr

		%is_of_char_comp = icmp eq i8 %str_char, %counting_char

		br i1 %is_of_char_comp, label %continue, label %exit

	continue:
		%next_i = add i32 %i, 1

		%at_end_comp = icmp eq i32 %i, %lower_bound

		br i1 %at_end_comp, label %exit, label %loop

	exit:
		%ret_val = sub i32 %upper_bound, %i

		ret i32 %ret_val
}