FROM rust:1.81

# Create app directory
WORKDIR /app

# Copy source code
COPY . .

# Copy your file from local filesystem into the container
COPY container-config.json /app/src/config/config.json

# Build the Rust application
RUN cargo build --release

# Expose API port
EXPOSE 3005

# Run the application
CMD ["./target/release/btc-api"]
