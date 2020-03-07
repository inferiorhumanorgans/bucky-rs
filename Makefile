build:
	@cargo build --lib --tests --examples

release:
	@cargo build --release --lib --examples

test:
	@cargo test tests --lib --examples

bench:
	@cargo bench benches --lib --examples

docs:
	@cargo doc --lib --no-deps
