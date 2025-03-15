# --- Build Stage ---
FROM rust:latest AS builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    musl-dev \
    musl-tools \
    libssl-dev \
    pkg-config \
    build-essential \
    curl \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Rust target for musl-based static linking
RUN rustup target add x86_64-unknown-linux-musl

# Install musl cross toolchain
RUN curl -LO https://musl.cc/x86_64-linux-musl-cross.tgz && \
    tar -xzf x86_64-linux-musl-cross.tgz && \
    mv x86_64-linux-musl-cross /opt/musl

# Set environment variables for musl toolchain
ENV PATH="/opt/musl/bin:$PATH"
ENV CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc

# Set working directory
WORKDIR /app

# Set OpenSSL environment variables
ENV OPENSSL_DIR=/usr
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_STATIC=true

# Copy source code
COPY . .

# Build the application with musl target
RUN cargo build --release --target=x86_64-unknown-linux-musl

# --- Runtime Stage ---
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libssl3

# Set working directory
WORKDIR /app

# Copy your file from local filesystem into the container
COPY container-config.json /app/src/config/config.json

# Copy the compiled binary from builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/btc-api /app/

# Ensure binary is executable
RUN chmod +x /app/btc-api

# Expose the correct port for Google Cloud Run
ENV PORT=8080
EXPOSE 8080

# Start the application
CMD ["/app/btc-api"]
