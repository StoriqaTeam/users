build:
	@cargo build --release

clean:
	@cargo clean

run:
	./target/release/users

.PHONY: build clean run