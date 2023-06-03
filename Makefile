coverage:
	cargo llvm-cov --ignore-filename-regex 'main.rs'
	cargo llvm-cov report --lcov --ignore-filename-regex 'main.rs' --output-path lcov.info

test:
	cargo test

test-log:
	cargo test -- --nocapture
