jit: target/build.ll
	lli $^ main.tower

target/build.ll: src/main.ll src/lexer.ll src/str_utils.ll
	@mkdir -p target
	llvm-link -S -o target/build.ll $^