build:
	@cargo build --release

clean:
	@cargo clean

run:
	@RUST_LOG=info ./target/release/users

docker:
	@docker-compose -f docker/docker-compose.yml run users

.PHONY: build clean run
