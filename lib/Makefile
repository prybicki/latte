all:
	clang -c -emit-llvm runtime.c
	clang -S -emit-llvm runtime.c
clean:
	rm runtime.ll
	rm runtime.bc

