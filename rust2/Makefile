.PHONY: all
all: build

STEPS := step0_repl

$(STEPS): build
	cp target/release/$@ $@

.PHONY: build
build:
	nix-shell --run 'cargo build --release'

.PHONY: clean
clean:
	nix-shell --run 'cargo clean'
	rm $(STEPS)
