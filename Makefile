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

# SP1
prove_sp1_fibonacci:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/fibonacci --enable-telemetry

prove_sp1_rsa:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/rsa

prove_sp1_ecdsa:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/ecdsa
	
prove_sp1_json:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/json

prove_sp1_regex:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/regex

prove_sp1_sha:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/sha

prove_sp1_tendermint:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/tendermint

prove_sp1_zkquiz:
	@RUST_LOG=info cargo run --release -- prove-sp1 examples/zkquiz

# Docker commands
docker-shell:
	docker run -it \
		-v zkrust-cargo-registry:/root/.cargo/registry \
		-v zkrust-cargo-git:/root/.cargo/git \
		-v "$(PWD)/src:/zkrust/src" \
		-w /zkrust \
		zkrust bash

docker-build:
	docker build --platform=linux/amd64 -t zkrust .