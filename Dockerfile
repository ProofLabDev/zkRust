# Use Ubuntu as base image
FROM ubuntu:22.04

# Prevent timezone prompt during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install essential packages
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    clang \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install nightly toolchain
RUN rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rustfmt clippy

# Create working directory
WORKDIR /zkrust

# Copy project files
COPY install_zkrust_from_source.sh /zkrust/install_zkrust_from_source.sh
COPY examples /zkrust/examples
COPY Makefile /zkrust/Makefile
COPY src /zkrust/src
COPY workspaces /zkrust/workspaces
COPY zk_rust_io /zkrust/zk_rust_io
COPY rust-toolchain.toml /zkrust/rust-toolchain.toml
COPY Cargo.toml /zkrust/Cargo.toml
COPY Cargo.lock /zkrust/Cargo.lock

# Set shell to bash for install script
SHELL ["/bin/bash", "-c"]

# Install zkRust and its dependencies
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    chmod +x /zkrust/install_zkrust_from_source.sh && \
    bash /zkrust/install_zkrust_from_source.sh && \
    echo 'source ~/.bashrc' >> ~/.bash_profile && \
    echo 'source ~/.bashrc' >> ~/.profile

# Set environment variables
ENV ZKRUST_DIR=/root/.zkRust
ENV ZKRUST_BIN_DIR=/root/.zkRust/bin
ENV PATH="${PATH}:${ZKRUST_BIN_DIR}"

# Create entrypoint script properly
RUN printf '#!/bin/bash\nsource ~/.bashrc\nexec "$@"\n' > /entrypoint.sh && \
    cat /entrypoint.sh && \
    chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["bash"] 