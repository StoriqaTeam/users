build:
	@cargo build --release

clean:
	@cargo clean

run:
	@RUST_LOG=info ./target/release/users

docker:
	@docker build -t users .

docker-run:
	@docker run -it --rm users

.PHONY: build clean run docker docker-run
