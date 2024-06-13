# Build stage
FROM rust:latest AS builder
SHELL ["/bin/bash", "-c"]

WORKDIR /usr/src/myapp

# Install dependencies
RUN apt-get update && \
    apt-get install -y \
        curl \
        git \
        build-essential \
        libssl-dev \
        pkg-config && \
    rm -rf /var/lib/apt/lists/*

# Install Rust tools
RUN rustup toolchain install nightly-2023-12-16-x86_64 && \
    rustup default nightly-2023-12-16 && \
    rustup component add rust-src --toolchain nightly-2023-12-16-x86_64

# Install Rust dependencies
RUN cargo install \
        cargo-dylint \
        dylint-link \
        mdbook \
        cargo-scout-audit

COPY entrypoint.sh /usr/src/entrypoint.sh
RUN chmod +x /usr/src/entrypoint.sh

# Final stage
FROM rust:slim

# Install necessary runtime dependencies
RUN apt-get update && \
    apt-get install -y \
        bash \
        curl \
        openssl \
        ca-certificates \
        libstdc++6 && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m myuser

WORKDIR /usr/src/myapp

# Copy entrypoint script and installed tools from the builder stage
COPY --from=builder /usr/src/entrypoint.sh /usr/src/entrypoint.sh
COPY --from=builder /usr/local/cargo/bin/cargo-scout-audit /usr/local/cargo/bin/cargo-scout-audit

# Ensure the script and binary have the correct permissions
RUN chmod +x /usr/src/entrypoint.sh /usr/local/cargo/bin/cargo-scout-audit

# Run as non-root user
RUN chown -R myuser:myuser /usr/src/myapp
USER myuser

VOLUME /scoutme
ENTRYPOINT ["/usr/src/entrypoint.sh"]