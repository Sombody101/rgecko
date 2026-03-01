.PHONY: release debug test fuzz

release:
	cargo build --release

debug:
	cargo build

test:
	cargo test

fuzz:
	# Fuzz doesn't have a way to choose a profile, so a manual LTO is needed.
	# And, cherry on the cake, you have to use a rustc nightly
	CARGO_PROFILE_RELEASE_LTO=false cargo +nightly fuzz run fuzz_target_1