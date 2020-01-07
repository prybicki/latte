all:
	cargo build --release
	cp ./target/release/latte ./latc_llvm
clean:
	cargo clean
	rm ./latc_llvm

