all:
	clang -O2 -c -emit-llvm runtime.c
	clang -O2 -S -emit-llvm runtime.c
clean:
	rm runtime.ll
	rm runtime.bc

