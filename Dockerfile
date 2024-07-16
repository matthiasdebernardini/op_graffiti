ARG RUST_VERSION=1.79.0
ARG APP_NAME=op_grafitti
ARG BITCOIN_CORE_VERSION=0.21.0
ARG ELECTRS_VERSION=0.10.5

# Use a multi-arch base image for building
FROM --platform=$TARGETPLATFORM rust:${RUST_VERSION}-bullseye AS build

ARG BITCOIN_CORE_VERSION
ARG APP_NAME
ARG ELECTRS_VERSION
WORKDIR /app

RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    cmake \
    git \
    libssl-dev \
    libevent-dev \
    libboost-system-dev \
    libboost-filesystem-dev \
    libboost-chrono-dev \
    libboost-test-dev \
    libboost-thread-dev \
    libdb++-dev \
    libsqlite3-dev \
    libgflags-dev \
    libsnappy-dev \
    zlib1g-dev \
    libbz2-dev \
    liblz4-dev \
    libzstd-dev \
    bsdmainutils

# Build Bitcoin Core
RUN git clone https://github.com/bitcoin/bitcoin.git \
    && cd bitcoin \
    && git checkout v${BITCOIN_CORE_VERSION} \
    && ./autogen.sh \
    && ./configure --without-gui --without-miniupnpc --with-incompatible-bdb \
    && make -j$(nproc) \
    && make install

# Build RocksDB
RUN git clone -b v7.8.3 --depth 1 https://github.com/facebook/rocksdb \
    && cd rocksdb \
    && make shared_lib -j $(nproc) \
    && make install-shared

# Build Electrs
RUN git clone https://github.com/romanz/electrs.git \
    && cd electrs \
    && git checkout v${ELECTRS_VERSION} \
    && ROCKSDB_INCLUDE_DIR=/usr/local/include ROCKSDB_LIB_DIR=/usr/local/lib cargo build --locked --release

# Build the Rust application with optimized caching
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

# Start a new stage for the final image
FROM --platform=$TARGETPLATFORM debian:bullseye-slim AS final

ARG BITCOIN_CORE_VERSION
ARG APP_NAME

# Create a non-privileged user
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    appuser

# Copy the built binaries from the build stage
COPY --from=build /usr/local/bin/bitcoind /usr/local/bin/bitcoind
COPY --from=build /bin/server /bin/server
COPY --from=build /app/electrs/target/release/electrs /usr/local/bin/electrs
COPY --from=build /usr/local/lib/librocksdb.so* /usr/local/lib/
COPY --from=build /usr/lib/aarch64-linux-gnu/libgflags.so* /usr/lib/aarch64-linux-gnu/

# Set environment variables for Bitcoin Core and Electrs
ENV BITCOIND_EXE=/usr/local/bin/bitcoind
ENV ELECTRS_EXE=/usr/local/bin/electrs

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl1.1 \
    libevent-2.1-7 \
    libevent-pthreads-2.1-7 \
    libboost-system1.74.0 \
    libboost-filesystem1.74.0 \
    libboost-chrono1.74.0 \
    libboost-thread1.74.0 \
    libdb5.3++ \
    libsqlite3-0 \
    libsnappy1v5 \
    liblz4-1 \
    libzstd1 \
    libgflags2.2 \
    bsdmainutils \
    && rm -rf /var/lib/apt/lists/*

# Update library cache
RUN ldconfig

# Add commands to check library dependencies and versions
RUN ldd $BITCOIND_EXE && $BITCOIND_EXE --version
RUN ldd $ELECTRS_EXE && $ELECTRS_EXE --version

# Create necessary directories and set permissions
RUN mkdir -p /home/appuser/.bitcoin /home/appuser/db && \
    chown -R appuser:appuser /home/appuser/.bitcoin /home/appuser/db /usr/local/bin/electrs

# Copy the start script
COPY start.sh /start.sh
RUN chmod +x /start.sh

# Switch to non-root user
USER appuser

EXPOSE 8332 8333 18443 18444 50001 60401 9000

CMD ["/start.sh"]