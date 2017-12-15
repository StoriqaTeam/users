build:
	@cargo build --release

clean:
	@cargo clean

run:
	@RUST_LOG=info ./target/release/users

.PHONY: build clean run