; ModuleID = 'builtin'
source_filename = "builtin"

declare i8* @malloc(i64 %size)
declare i8* @realloc(i8* %ptr, i64 %size)
declare void @free(i8* %ptr)
declare i32 @puts(i8* %ptr)

declare void @tower_main(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr)

@hello_world_str = private constant [20 x i8] c"\e2\9c\a8 hello world \e2\9c\a8\00"

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

	call void @free(i8* %bp)

	ret i32 0
}

define void @__internal_spsub(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 %dec) {
	%bp = load i8*, i8** %bp_ptr
	%sp = load i8*, i8** %sp_ptr
	%ep = load i8*, i8** %ep_ptr

	%spi = ptrtoint i8* %sp to i64
	%spi_new = sub i64 %spi, %dec
	%sp_new = inttoptr i64 %spi_new to i8*

	store i8* %sp_new, i8** %sp_ptr

	ret void
}

define void @__internal_spadd(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr, i64 %inc) {
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

define void @__hello(i8** %bp_ptr, i8** %sp_ptr, i8** %ep_ptr) {
	%strp = getelementptr [20 x i8], [20 x i8]* @hello_world_str, i32 0, i32 0
	call i32 @puts(i8* %strp)

	ret void
}