.PHONY: tdd
tdd:
	watchexec --clear "time cargo test $(only) -q -- --nocapture"

.PHONY: bench
bench:
	cargo bench -- $(only)

.PHONY: test
test:
	cargo test $(only)

.PHONY: clean
clean:
	cargo clean
