build:
	@cargo build --release

clean:
	@rm -rf target/

run:
	./target/release/users

format:
	@cargo fmt

doc:
	@cargo doc
	@open ./target/doc/users/index.html

.PHONY: build clean run format doc
