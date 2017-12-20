build:
	@cargo build --release

clean:
	@cargo clean

run:
	@RUST_LOG=info ./target/release/users

docker:
	@docker build .

compose:
	@docker-compose up

.PHONY: build clean run docker compose
