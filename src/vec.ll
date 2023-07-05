; TODO: Problem: This is not generic. I kinda do want like a Vec type, but I'd want more of a generic one...
;       Maybe I can write a li'l preprocessor or macro engine or whatnot to do some monomorphisation... but then I'm not really writing LLVM IR any more
;       I could just copy-paste and modify it for each type I wanna support

%Vec = type {
	i64, ; length
	i64, ; capacity
	i8*
}

define %Vec @Vec.new() {
	%vec_ptr = alloca %Vec

	%len_ptr = bitcast %Vec* %vec_ptr to i64*
	store i64 0, i64* %len_ptr
	%capa_ptr = getelementptr inbounds %Vec, %Vec* %vec_ptr, i32 0, i32 1
	store i64 0, i64* %capa_ptr
	%data_ptr = getelementptr inbounds %Vec, %Vec* %vec, i32 0, i32 2
	store i8* null, i8** %data_ptr

	%vec = load %Vec, %Vec* %vec_ptr

	ret %Vec %vec
}

; define void @Vec.realloc(%Vec %vec) {

; }

define i64 @Vec.len(%Vec* %vec) {
	%len_ptr = bitcast %Vec* %vec to i64*
	%len = load i64, i64* %len_ptr

	ret i64 %len
}

define i64 @Vec.capacity(%Vec* %vec) {
	%capacity_ptr = getelementptr inbounds %Vec, %Vec* %vec, i32 0, i32 1
	%capacity = load i64, i64* %capacity_ptr

	ret i64 %capacity
}

define i8* @Vec.data(%Vec* %vec) {
	%data_ptr = getelementptr inbounds %Vec, %Vec* %vec, i32 0, i32 2
	%data = load i8*, i8** %data_ptr

	ret i8* %data
}

define void @Vec.drop(%Vec* %vec) {
	%data = call i8* @Vec.data(%Vec %vec)
	call void @free(%i8* %data)

	ret void
}