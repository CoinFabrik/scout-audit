# Stage 1: Builder
# Using the latest Rust image to set up the build environment
FROM rust:1.84 AS builder
SHELL ["/bin/bash", "-c"]

# Copy and set permissions for the entrypoint script
COPY entrypoint.sh /usr/src/scout/entrypoint.sh

# Copy local cargo-scout-audit project files
COPY / /usr/src/scout-audit

# Install cargo-scout-audit from the local path
WORKDIR /usr/src/scout-audit/apps/cargo-scout-audit
RUN cargo install --features docker_container --path . --locked
WORKDIR /usr/src/scout-audit/detectors/ink
RUN cargo +nightly-2024-07-11 build --release
WORKDIR /usr/src/scout-audit/detectors/rust
RUN cargo +nightly-2024-07-11 build --release
WORKDIR /usr/src/scout-audit/detectors/soroban
RUN cargo +nightly-2024-07-11 build --release
WORKDIR /usr/src/scout-audit/detectors/substrate-pallets
RUN cargo +nightly-2024-07-11 build --release

# Stage 2: Final
# Base image with Rust slim version for the runtime environment
FROM rust:1.84 AS final

# Install only necessary runtime dependencies
RUN apt-get update && apt-get install -y libcurl4 libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

# Copy the .rustup directory from the builder stage
COPY --from=builder /usr/local/rustup /usr/local/rustup
ENV PATH="/usr/local/rustup/bin:$PATH"

# Copy necessary binaries from the builder stage
COPY --from=builder /usr/local/cargo/bin/cargo-scout-audit /usr/local/cargo/bin/
COPY --from=builder /usr/local/cargo/bin/dylint-link /usr/local/cargo/bin/
COPY --from=builder /usr/src/scout/entrypoint.sh /usr/local/bin/
COPY --from=builder /usr/src/scout-audit /scout-audit

# Ensure the script and binaries are executable
RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/cargo/bin/*

# Define volume for application data
VOLUME /scoutme

# Set the entrypoint to the script
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
