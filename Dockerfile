FROM oven/bun:1 AS client-build
WORKDIR /client
COPY apps/client/package.json apps/client/bun.lock* ./
RUN bun install --frozen-lockfile
COPY apps/client/ .
RUN bun run build

FROM rust:1-bookworm AS api-build
WORKDIR /repo
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY apps/api/Cargo.toml apps/api/build.rs apps/api/
COPY apps/cli/Cargo.toml apps/cli/
COPY crates/scene/Cargo.toml crates/scene/
RUN mkdir -p apps/api/src apps/api/migrations apps/cli/src crates/scene/src \
    && echo 'fn main() {}' > apps/api/src/main.rs && echo '' > apps/api/src/lib.rs && echo '' > crates/scene/src/lib.rs \
    && echo 'fn main() {}' > apps/cli/src/main.rs && echo '' > apps/cli/src/lib.rs \
    && cargo build --release --locked -p api && rm -rf apps/api/src apps/api/migrations crates/scene/src
COPY crates crates
COPY docs/format.md docs/format.md
COPY apps/api apps/api
COPY apps/client/package.json /version/package.json
COPY scripts/version.sh /version/version.sh
RUN touch apps/api/src/main.rs apps/api/src/lib.rs crates/scene/src/lib.rs \
    && APP_VERSION="$(sh /version/version.sh /version/package.json)" && export APP_VERSION \
    && cargo build --release --locked -p api

FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f
COPY --from=api-build /repo/target/release/api /api
COPY --from=client-build /client/build /client
ENV CLIENT_DIR=/client
ENV PORT=4000
ENV DATA_DIR=/data
EXPOSE 4000
HEALTHCHECK --interval=10s --timeout=5s --retries=3 CMD ["/api", "healthcheck"]
USER nonroot:nonroot
ENTRYPOINT ["/api"]
