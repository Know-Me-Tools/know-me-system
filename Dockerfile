# syntax=docker/dockerfile:1.18@sha256:dabfc0969b935b2080555ace70ee69a5261af8a8f1b4df97b9e7fbcf6722eddf

FROM node:24-bookworm-slim@sha256:6f7b03f7c2c8e2e784dcf9295400527b9b1270fd37b7e9a7285cf83b6951452d AS web
WORKDIR /workspace
RUN corepack enable && corepack prepare pnpm@11.11.0 --activate
COPY packages ./packages
COPY rust/crates/tauri-plugin-gen-ui/guest-js ./rust/crates/tauri-plugin-gen-ui/guest-js
COPY desktop/package.json desktop/pnpm-lock.yaml desktop/pnpm-workspace.yaml ./desktop/
WORKDIR /workspace/desktop
RUN pnpm install --frozen-lockfile
COPY desktop ./
RUN pnpm build

FROM rust:1.96-trixie@sha256:1f0dbad1df66647807e6952d1db85d0b2bda7606cb2139d82517e4f009967376 AS rust-builder
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        clang cmake git libclang-dev pkg-config protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace/rust
COPY rust ./
COPY --from=web /workspace/desktop/dist /web-dist
ENV KNOWME_WEB_DIST_DIR=/web-dist
RUN cargo build --locked --release -p knowme-web-server

FROM debian:trixie-slim@sha256:020c0d20b9880058cbe785a9db107156c3c75c2ac944a6aa7ab59f2add76a7bd AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl libstdc++6 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 knowme \
    && useradd --uid 10001 --gid knowme --create-home --shell /usr/sbin/nologin knowme \
    && install -d -o knowme -g knowme /var/lib/knowme
COPY --from=rust-builder /workspace/rust/target/release/knowme-web-server /usr/local/bin/knowme-web-server
USER 10001:10001
ENV KNOWME_BIND=0.0.0.0:8080 \
    KNOWME_DATA_DIR=/var/lib/knowme \
    RUST_LOG=info
VOLUME ["/var/lib/knowme"]
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=30s --retries=5 \
    CMD ["curl", "--fail", "--silent", "http://127.0.0.1:8080/api/v1/ready"]
ENTRYPOINT ["/usr/local/bin/knowme-web-server"]
