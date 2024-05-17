FROM rust:latest
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
RUN rustup toolchain install nightly-2023-12-16-x86_64-unknown-linux-gnu && \
    rustup default nightly-2023-12-16 && \
    rustup component add rust-src --toolchain nightly-2023-12-16-x86_64-unknown-linux-gnu

# Install Rust dependencies
RUN cargo install \
        cargo-dylint \
        dylint-link \
        mdbook \
        cargo-scout-audit

COPY entrypoint.sh /usr/src/entrypoint.sh
RUN chmod +x /usr/src/entrypoint.sh

VOLUME /scoutme
ENTRYPOINT ["/usr/src/entrypoint.sh"]
