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
COPY apps/worker/Cargo.toml apps/worker/
COPY crates/scene/Cargo.toml crates/scene/
COPY crates/capture/Cargo.toml crates/capture/
RUN mkdir -p apps/api/src apps/api/migrations apps/cli/src apps/worker/src crates/scene/src crates/capture/src \
    && echo 'fn main() {}' > apps/api/src/main.rs && echo '' > apps/api/src/lib.rs && echo '' > crates/scene/src/lib.rs \
    && echo 'fn main() {}' > apps/cli/src/main.rs && echo '' > apps/cli/src/lib.rs \
    && echo 'fn main() {}' > apps/worker/src/main.rs && echo '' > crates/capture/src/lib.rs \
    && cargo build --release --locked -p api -p worker \
    && rm -rf apps/api/src apps/api/migrations apps/worker/src crates/scene/src crates/capture/src
COPY crates crates
COPY docs/format.md docs/format.md
COPY apps/api apps/api
COPY apps/worker apps/worker
COPY apps/client/package.json /version/package.json
COPY scripts/version.sh /version/version.sh
RUN touch apps/api/src/main.rs apps/api/src/lib.rs apps/worker/src/main.rs crates/scene/src/lib.rs \
       crates/capture/src/lib.rs \
    && APP_VERSION="$(sh /version/version.sh /version/package.json)" && export APP_VERSION \
    && cargo build --release --locked -p api -p worker

FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc AS chrome
ARG CHROME_VERSION=157.0.8103.78
ARG CHROME_SHA256=f90329720fb61e5a76da9b6b509b6ad12a484c9c5df0359403e935177c6d153a
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl unzip \
    && rm -rf /var/lib/apt/lists/* \
    && curl -fsSL -o /tmp/chrome.zip \
       "https://storage.googleapis.com/chrome-for-testing-public/${CHROME_VERSION}/linux64/chrome-headless-shell-linux64.zip" \
    && echo "${CHROME_SHA256}  /tmp/chrome.zip" | sha256sum -c - \
    && unzip -q /tmp/chrome.zip -d /opt && rm /tmp/chrome.zip

FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates tini \
       fonts-liberation fonts-noto-color-emoji libasound2 libatk-bridge2.0-0 libatk1.0-0 libatspi2.0-0 \
       libdbus-1-3 libexpat1 libgbm1 libglib2.0-0 libnspr4 libnss3 libudev1 libx11-6 libxcb1 libxcomposite1 \
       libxdamage1 libxext6 libxfixes3 libxkbcommon0 libxrandr2 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=chrome /opt/chrome-headless-shell-linux64 /opt/chrome-headless-shell
COPY --from=api-build /repo/target/release/api /api
COPY --from=api-build /repo/target/release/worker /worker
COPY --from=client-build /client/build /client
ENV CLIENT_DIR=/client
ENV PORT=4000
ENV DATA_DIR=/data
ENV HOME=/tmp
ENV DIAPO_CHROME=/opt/chrome-headless-shell/chrome-headless-shell
EXPOSE 4000
HEALTHCHECK --interval=10s --timeout=5s --retries=3 CMD ["/api", "healthcheck"]
USER 65532:65532
ENTRYPOINT ["/usr/bin/tini", "--", "/api"]
