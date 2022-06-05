FROM rustlang/rust:nightly-bullseye AS chef

### 2. Copy the files in your machine to the Docker image
##COPY ./ /app
##WORKDIR /app
##
### Build your program for release
##RUN cargo build --bin trade-hostd --release
##
##EXPOSE 4002

### Run the binary
##CMD ["./target/release/trade-hostd"]
# We only pay the installation cost once, 
# it will be cached from the second build onwards
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin trade-hostd

# We do not need the Rust toolchain to run the binary!
FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt update && apt install -y --no-install-recommends \
    ca-certificates \
    curl \
    gnupg2 \
    libcurl4 libcurl4-openssl-dev \
    software-properties-common
COPY --from=builder /app/target/release/trade-hostd /usr/local/bin
EXPOSE 4002
ENTRYPOINT ["/usr/local/bin/trade-hostd"]