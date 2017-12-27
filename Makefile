build:
	@cargo build --release

clean:
	@cargo clean

doc:
	@cargo doc --no-deps --open

run:
	@RUST_LOG=info ./target/release/users

docker:
	@docker build -t users .

compose:
	@docker-compose up

.PHONY: build clean doc run docker compose
