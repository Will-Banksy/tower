jit: target/build.ll
	lli $^ main.tower

target/build.ll: src/main.ll
	@mkdir -p target
	llvm-link -o target/build.ll $^