; BEGIN STUB

@println_str_fmt_str = private constant [4 x i8] c"%s\0a\00"
@println_uint_fmt_str = private constant [4 x i8] c"%u\0a\00"
@dbg_checkpoint_str = private constant [15 x i8] c"Checkpoint %u\0a\00"

declare i32 @printf(i8* %fmt_str, ...) ; format string, ...arguments -> error code
declare i8* @malloc(i64 %size)
declare i8* @realloc(i8* %ptr, i64 %size)
declare i32 @puts(i8* %str)

define i32 @main(i32 %argc, i8** %argv) {
	%init_size = add i32 4096, 0
	%init_size_ptr = inttoptr i32 %init_size to i8*
	; Base ptr
	%bp = call i8* @malloc(i32 %init_size)
	; Stack ptr
	%sp = getelementptr i8, i8* %bp, i64 0
	; End ptr
	%ep = getelementptr i8, i8* %bp, i64 4096

	%bp_ptr = alloca i8*
	store i8* %bp, i8** %bp_ptr
	%sp_ptr = alloca i8*
	store i8* %sp, i8** %sp_ptr
	%ep_ptr = alloca i8*
	store i8* %ep, i8** %ep_ptr

	call void @tower_main(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr)

	ret i32 0
}

define void @__dbg_checkpoint(i32 %num) {
	%dbg_checkpoint_strp = getelementptr [15 x i8], [15 x i8]* @dbg_checkpoint_str, i64 0, i64 0
	%printf_ret = call i32 (i8*, ...) @printf(i8* %dbg_checkpoint_strp, i32 %num)

	ret void
}

define void @__dbg_print_ppval(i8** %p) {
	%val = load i8*, i8** %p
	%println_uint_fmt_strp = getelementptr [4 x i8], [4 x i8]* @println_uint_fmt_str, i64 0, i64 0
	%printf_ret = call i32 (i8*, ...) @printf(i8* %println_uint_fmt_strp, i8* %val)

	ret void
}

define void @__dbg_print_ptr(i8* %val) {
	%println_uint_fmt_strp = getelementptr [4 x i8], [4 x i8]* @println_uint_fmt_str, i64 0, i64 0
	%printf_ret = call i32 (i8*, ...) @printf(i8* %println_uint_fmt_strp, i8* %val)

	ret void
}

define void @__dbg_print_val(i64 %val) {
	%println_uint_fmt_strp = getelementptr [4 x i8], [4 x i8]* @println_uint_fmt_str, i64 0, i64 0
	%printf_ret = call i32 (i8*, ...) @printf(i8* %println_uint_fmt_strp, i64 %val)

	ret void
}

define void @__spsub(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 %dec) {
	%bp = load i8*, i8** %bp_ptr
	%sp = load i8*, i8** %sp_ptr
	%ep = load i8*, i8** %ep_ptr

	%spi = ptrtoint i8* %sp to i64
	%spi_new = sub i64 %spi, %dec
	%sp_new = inttoptr i64 %spi_new to i8*

	store i8* %sp_new, i8** %sp_ptr

	ret void
}

define void @__spadd(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 %inc) {
	%bp = load i8*, i8** %bp_ptr
	%sp = load i8*, i8** %sp_ptr
	%ep = load i8*, i8** %ep_ptr

	%spi = ptrtoint i8* %sp to i64
	%epi = ptrtoint i8* %ep to i64
	%space_left = sub i64 %epi, %spi
	%enough_space = icmp ugt i64 %space_left, %inc

	br i1 %enough_space, label %spadd-ret, label %spadd-realloc

spadd-realloc:
	%bpi = ptrtoint i8* %bp to i64
	%current_size = sub i64 %epi, %bpi
	%new_size = mul i64 %current_size, 2
	%spi_offset = sub i64 %spi, %bpi
	%spi_offset_new = add i64 %spi_offset, %inc

	%bp_new = call i8* @realloc(i8* %bp, i64 %new_size)
	%sp_new = getelementptr i8, i8* %bp_new, i64 %spi_offset_new
	%ep_new = getelementptr i8, i8* %bp_new, i64 %new_size

	store i8* %bp_new, i8** %bp_ptr
	store i8* %sp_new, i8** %sp_ptr
	store i8* %ep_new, i8** %ep_ptr
	ret void

spadd-ret:
	%sp_new_2 = getelementptr i8, i8* %sp, i64 %inc
	store i8* %sp_new_2, i8** %sp_ptr
	ret void
}

define void @__println_str(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr) {
	call void @__spsub(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 8)
	%bp = load i8*, i8** %bp_ptr
	%sp = load i8*, i8** %sp_ptr
	%ep = load i8*, i8** %ep_ptr
	%strp = load i8*, i8* %sp

	; %println_uint_fmt_strp = getelementptr [4 x i8], [4 x i8]* @println_uint_fmt_str, i64 0, i64 0
	; %printf_ret_dbg = call i32 (i8*, ...) @printf(i8* %println_uint_fmt_strp, i8* %strp)

	%println_str_fmt_strp = getelementptr [4 x i8], [4 x i8]* @println_str_fmt_str, i64 0, i64 0
	%printf_ret = call i32 (i8*, ...) @printf(i8* %println_str_fmt_strp, i8* %strp)

	ret void
}

define void @__println_u32(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr) {
	call void @__spsub(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 4)
	%bp = load i8*, i8** %bp_ptr
	%sp = load i8*, i8** %sp_ptr
	%ep = load i8*, i8** %ep_ptr
	%val = load i32, i32* %sp

	%println_uint_fmt_strp = getelementptr [4 x i8], [4 x i8]* @println_uint_fmt_str, i64 0, i64 0
	%printf_ret = call i32 (i8*, ...) @printf(i8* %println_uint_fmt_strp, i32 %val)

	ret void
}

; END STUB

@anon_str_xyz = private constant [20 x i8] c"\e2\9c\a8 hello world \e2\9c\a8\00"

%Point = type {
	i32, i32
}

define void @tower_main(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr) {
	%bp = load i8*, i8** %bp_ptr
	%sp = load i8*, i8** %sp_ptr
	%ep = load i8*, i8** %ep_ptr

	; Push str pointer to stack
	%anon_str_xyz_p = getelementptr [20 x i8], [20 x i8]* @anon_str_xyz, i64 0, i64 0
	store i8* %anon_str_xyz_p, i8* %sp
	call void @__spadd(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 8)
	%bp.1 = load i8*, i8** %bp_ptr
	%sp.1 = load i8*, i8** %sp_ptr
	%ep.1 = load i8*, i8** %ep_ptr

	; call void @__spsub(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 8)

	; %bp.2 = load i8*, i8** %bp_ptr
	; %sp.2 = load i8*, i8** %sp_ptr
	; %ep.2 = load i8*, i8** %ep_ptr

	; %anon_str_xyz_p_2 = load i8*, i8* %sp.2

	; Print the values of %sp after each operation
	; %println_uint_fmt_strp = getelementptr [4 x i8], [4 x i8]* @println_uint_fmt_str, i64 0, i64 0
	; %printf_ret = call i32 (i8*, ...) @printf(i8* %println_uint_fmt_strp, i8* %sp)
	; %printf_ret_2 = call i32 (i8*, ...) @printf(i8* %println_uint_fmt_strp, i8* %sp.1)
	; %printf_ret_3 = call i32 (i8*, ...) @printf(i8* %println_uint_fmt_strp, i8* %sp.2)

	; %println_str_fmt_strp = getelementptr [3 x i8], [3 x i8]* @println_str_fmt_str, i64 0, i64 0
	; %printf_ret_4 = call i32 (i8*, ...) @printf(i8* %println_uint_fmt_strp, i8* %anon_str_xyz_p_2)

	call void @ps(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr)
	%bp.2 = load i8*, i8** %bp_ptr
	%sp.2 = load i8*, i8** %sp_ptr
	%ep.2 = load i8*, i8** %ep_ptr

	store i32 0, i8* %sp.2
	call void @__spadd(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 4)
	%bp.3 = load i8*, i8** %bp_ptr
	%sp.3 = load i8*, i8** %sp_ptr
	%ep.3 = load i8*, i8** %ep_ptr

	store i32 1, i8* %sp.3
	call void @__spadd(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 4)
	%bp.4 = load i8*, i8** %bp_ptr
	%sp.4 = load i8*, i8** %sp_ptr
	%ep.4 = load i8*, i8** %ep_ptr

	; Get two args for making struct
	call void @__spsub(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 4)
	%bp.5 = load i8*, i8** %bp_ptr
	%sp.5 = load i8*, i8** %sp_ptr
	%ep.5 = load i8*, i8** %ep_ptr
	%struct_arg_1 = load i32, i8* %sp.5

	call void @__spsub(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 4)
	%bp.6 = load i8*, i8** %bp_ptr
	%sp.6 = load i8*, i8** %sp_ptr
	%ep.6 = load i8*, i8** %ep_ptr
	%struct_arg_0 = load i32, i8* %sp.6

	; Make and push struct
	%struct_val.0 = insertvalue %Point undef, i32 %struct_arg_0, 0
	%struct_val.1 = insertvalue %Point %struct_val.0, i32 %struct_arg_1, 1
	store %Point %struct_val.1, i8* %sp.6
	%struct_size_p = getelementptr %Point, %Point* null, i64 1 ; Trick with using GEP to calculate the size of an aggregate
	%struct_size = ptrtoint i8* %struct_size_p to i64
	call void @__spadd(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 %struct_size)
	%bp.7 = load i8*, i8** %bp_ptr
	%sp.7 = load i8*, i8** %sp_ptr
	%ep.7 = load i8*, i8** %ep_ptr

	; Get element of struct
	call void @__spsub(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 %struct_size)
	%bp.8 = load i8*, i8** %bp_ptr
	%sp.8 = load i8*, i8** %sp_ptr
	%ep.8 = load i8*, i8** %ep_ptr
	; %sp.8_as_pointp = bitcast i8* %sp.8 to %Point*
	%struct_val_1_p = getelementptr inbounds %Point, %Point* %sp.8, i32 0, i32 1
	%struct_val_1 = load i32, i32* %struct_val_1_p
	call void @__spadd(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 %struct_size)
	%bp.9 = load i8*, i8** %bp_ptr
	%sp.9 = load i8*, i8** %sp_ptr
	%ep.9 = load i8*, i8** %ep_ptr

	; Push element of struct
	store i32 %struct_val_1, i8* %sp.9
	call void @__spadd(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 4)
	%bp.10 = load i8*, i8** %bp_ptr
	%sp.10 = load i8*, i8** %sp_ptr
	%ep.10 = load i8*, i8** %ep_ptr

	call void @__println_u32(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr)
	%bp.11 = load i8*, i8** %bp_ptr
	%sp.11 = load i8*, i8** %sp_ptr
	%ep.11 = load i8*, i8** %ep_ptr

	ret void
}

define void @ps(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr) {
	call i8* @__println_str(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr)
	ret void
}