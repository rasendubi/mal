.PHONY: all
all: build

STEPS := step0_repl step1_read_print step2_eval step3_env

$(STEPS): build
	cp target/debug/$@ $@

.PHONY: build
build:
	nix-shell --run 'cargo build'

.PHONY: clean
clean:
	nix-shell --run 'cargo clean'
	rm $(STEPS)
