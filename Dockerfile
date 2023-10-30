FROM clux/muslrust:1.73.0 as chef

RUN apt-get update && apt-get install -y --no-install-recommends bzip2 gnupg && rm -rf /var/lib/apt/lists/*

# ENV TARGET "x86_64-unknown-linux-musl"
ENV TARGET x86_64-unknown-linux-gnu
ENV SYSTEM_DEPS_LINK static

ARG LIBGPG_ERROR_VER=1.47
WORKDIR /usr/src
ADD https://www.gnupg.org/ftp/gcrypt/libgpg-error/libgpg-error-${LIBGPG_ERROR_VER}.tar.bz2 ./
RUN tar -xjf libgpg-error-${LIBGPG_ERROR_VER}.tar.bz2
WORKDIR libgpg-error-$LIBGPG_ERROR_VER
RUN ./configure --host "$TARGET" --prefix="$PREFIX" --with-pic --enable-fast-install --disable-dependency-tracking --enable-static --disable-shared --disable-nls --disable-doc --disable-languages --disable-tests --enable-install-gpg-error-config
RUN make -j$(nproc) install

ARG LIBASSUAN_VER=2.5.6
WORKDIR /usr/src
ADD https://www.gnupg.org/ftp/gcrypt/libassuan/libassuan-${LIBASSUAN_VER}.tar.bz2 ./
RUN tar -xjf libassuan-${LIBASSUAN_VER}.tar.bz2
WORKDIR libassuan-$LIBASSUAN_VER
RUN ./configure --host "$TARGET" --prefix="$PREFIX" --with-pic --enable-fast-install --disable-dependency-tracking --enable-static --disable-shared --disable-doc --with-gpg-error-prefix="$PREFIX"
RUN make -j$(nproc) install

ARG GPGME_VER=1.23.1
WORKDIR /usr/src
ADD https://www.gnupg.org/ftp/gcrypt/gpgme/gpgme-${GPGME_VER}.tar.bz2 ./
RUN tar -xjf gpgme-${GPGME_VER}.tar.bz2
WORKDIR gpgme-$GPGME_VER
RUN ./configure --host "$TARGET" --prefix="$PREFIX" --with-pic --enable-fast-install --disable-dependency-tracking --enable-static --disable-shared --disable-languages --disable-gpg-test --with-gpg-error-prefix="$PREFIX" --with-libassuan-prefix="$PREFIX"
RUN make -j$(nproc) install
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target "$TARGET" --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY . .
RUN cargo build --release --target "$TARGET" --bin seiten

FROM debian:bookworm-slim AS runtime
ENV TARGET "x86_64-unknown-linux-gnu"
RUN apt-get update \
 && apt-get install --no-install-recommends -y libgpgme-dev \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/* \
 && apt-get autoremove -y
WORKDIR /app
COPY --from=builder /app/target/$TARGET/release/seiten .
CMD ["/app/seiten"]
