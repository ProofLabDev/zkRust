# SHELL := $(shell echo $$SHELL)

# # Find the profile directory based on the shell
# ifeq ($(findstring bash,$(SHELL)),bash)
#     PROFILE := ~/.bashrc
# else ifeq ($(findstring zsh,$(SHELL)),zsh)
#     PROFILE := ~/.zshenv
# else ifeq ($(findstring fish,$(SHELL)),fish)
#     PROFILE := ~/.config/fish/config.fish
# else ifeq ($(findstring sh,$(SHELL)),sh)
#     PROFILE := ~/.profile
# else
#     echo "zkrust: could not detect shell, manually add ${ZKRUST_BIN_DIR} to your PATH."
# 	exit 1
# endif

install: install_zkRust

install_zkRust:
	@curl -L https://raw.githubusercontent.com/yetanotherco/zkRust/main/install_zkrust.sh | bash

install_sp1:
	@curl -L https://sp1.succinct.xyz | bash
	@source $(PROFILE)
	@sp1up
	@cargo prove --version

install_risc0:
	@curl -L https://risczero.com/install | bash
	@rzup install
	@cargo risczero --version

all: install

__EXAMPLES__:

# RISC0
prove_risc0_fibonacci:
	@RUST_LOG=info cargo run --release -- prove-risc0 examples/fibonacci --enable-telemetry

prove_risc0_rsa:
	@RUST_LOG=info cargo run --release -- prove-risc0 examples/rsa

prove_risc0_ecdsa:
	@RUST_LOG=info cargo run --release -- prove-risc0 examples/ecdsa

prove_risc0_json:
	@RUST_LOG=info cargo run --release -- prove-risc0 examples/json

prove_risc0_regex:
	@RUST_LOG=info cargo run --release -- prove-risc0 examples/regex

prove_risc0_sha:
	@RUST_LOG=info cargo run --release -- prove-risc0 examples/sha

prove_risc0_tendermint:
	@RUST_LOG=info cargo run --release -- prove-risc0 examples/tendermint

prove_risc0_zkquiz:
	@RUST_LOG=info cargo run --release -- prove-risc0 examples/zkquiz

prove_risc0_bubble_sort:
	@RUST_LOG=info cargo run --release -- prove-risc0 examples/bubble_sort --enable-telemetry

# SP1
prove_sp1_fibonacci:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/fibonacci --enable-telemetry

prove_sp1_rsa:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/rsa --enable-telemetry --precompiles

prove_sp1_ecdsa:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/ecdsa --enable-telemetry --precompiles
	
prove_sp1_json:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/json

prove_sp1_regex:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/regex --enable-telemetry --precompiles

prove_sp1_sha:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/sha --enable-telemetry 

prove_sp1_tendermint:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/tendermint

prove_sp1_zkquiz:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/zkquiz

prove_sp1_iseven:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/is_even --enable-telemetry --precompiles

prove_sp1_bubble_sort:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/bubble_sort --enable-telemetry --precompiles

# Docker commands
docker-shell:
	docker run -it \
		-v zkrust-cargo-registry:/root/.cargo/registry \
		-v zkrust-cargo-git:/root/.cargo/git \
		-v "$(PWD)/src:/zkrust/src" \
		-v "$(PWD)/telemetry:/zkrust/telemetry" \
		-v "$(PWD)/Makefile:/zkrust/Makefile" \
		-v "$(PWD)/examples:/zkrust/examples" \
		-w /zkrust \
		zkrust bash

docker-build:
	docker build --platform=linux/amd64 -t zkrust .