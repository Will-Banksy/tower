interp:
	@mkdir -p target
	@find src -name "*.ll" | xargs llvm-link -o target/build.ll
	lli target/build.ll main.tower