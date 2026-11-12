# FORGE CLI + local web workspace. Build and run from the repository root:
#   docker build --build-arg FORGE_REVISION="$(git rev-parse HEAD)" -t forge:local .
#   docker run --rm forge:local --version
# Container usage and the workspace transport design: docs/container.md

FROM rust:1.93.0-slim-bookworm@sha256:776861219cd851131c1cec3bbd7cbeb16b99a794048097eb69ad9682a8ed0d57 AS build
WORKDIR /build
# Only the inputs the release binary embeds: schemas/, ui/workspace.{js,css},
# docs/api/*.openapi.yaml and src/mcp/*.json are include_str!-ed at compile time.
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY src src
COPY schemas schemas
# Bench targets are parsed by Cargo even for --bin; they never reach runtime.
COPY benches benches
COPY ui/workspace.js ui/workspace.css ui/
COPY docs/api docs/api
RUN cargo build --release --locked --bin forge

FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc AS runtime
# Fixed non-root identity; the workspace private-state qualification requires
# uid-owned directories with no group/other write bits (see docs/container.md).
RUN groupadd --gid 10001 forge \
    && useradd --uid 10001 --gid forge --home-dir /home/forge --create-home forge \
    && mkdir -p /home/forge/.local/state /project \
    && chown -R forge:forge /home/forge /project \
    && chmod 700 /home/forge/.local/state
COPY --from=build /build/target/release/forge /usr/local/bin/forge
ARG FORGE_REVISION=unspecified
LABEL org.opencontainers.image.title="forge" \
      org.opencontainers.image.description="Convert Markdown security policies into OSCAL artifacts; local single-user workspace." \
      org.opencontainers.image.source="https://github.com/policy-forge/forge" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.revision="${FORGE_REVISION}"
ENV HOME=/home/forge
USER forge
WORKDIR /project
ENTRYPOINT ["forge"]
CMD ["--help"]