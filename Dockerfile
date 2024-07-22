# Stage 1: Builder
# Using the latest Rust image to set up the build environment
FROM --platform=$TARGETPLATFORM rust:1.79 as builder
SHELL ["/bin/bash", "-c"]
WORKDIR /usr/src/scout

# Copy and set permissions for the entrypoint script
COPY entrypoint.sh /usr/src/scout/entrypoint.sh

# Copy local cargo-scout-audit project files
COPY /apps/cargo-scout-audit /usr/src/scout/cargo-scout-audit

# Install cargo-scout-audit from the local path
RUN cargo install --path /usr/src/scout/cargo-scout-audit --locked

# Stage 2: Final
# Base image with Rust slim version for the runtime environment
FROM --platform=$TARGETPLATFORM rust:1.79-slim as final

# Install only necessary runtime dependencies
RUN apt-get update && apt-get install -y libcurl4 && \
    rm -rf /var/lib/apt/lists/*

# Copy the .rustup directory from the builder stage
COPY --from=builder /usr/local/rustup /usr/local/rustup
ENV PATH="/usr/local/rustup/bin:$PATH"

# Copy necessary binaries from the builder stage
COPY --from=builder /usr/local/cargo/bin/cargo-scout-audit /usr/local/cargo/bin/
COPY --from=builder /usr/local/cargo/bin/dylint-link /usr/local/cargo/bin/
COPY --from=builder /usr/local/cargo/bin/cargo-dylint /usr/local/cargo/bin/
COPY --from=builder /usr/src/scout/entrypoint.sh /usr/local/bin/

# Ensure the script and binaries are executable
RUN chmod +x /usr/local/bin/entrypoint.sh /usr/local/cargo/bin/*

# Define volume for application data
VOLUME /scoutme

# Set the entrypoint to the script
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
