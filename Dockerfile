# Build the frontend and server, then ship a small runtime image.
FROM rust:1-bookworm AS build
RUN rustup target add wasm32-unknown-unknown \
 && curl -sSL https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz | tar xz -C /usr/local/bin
WORKDIR /app
COPY . .
RUN cd web && trunk build --release
RUN cargo build -p codesprout-server --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* && mkdir -p /data
WORKDIR /app
COPY --from=build /app/target/release/codesprout-server /app/codesprout-server
COPY --from=build /app/web/dist /app/web/dist
ENV ADDR=0.0.0.0:3000 DATABASE_PATH=/data/codesprout.db WEB_DIST=/app/web/dist
EXPOSE 3000
CMD ["/app/codesprout-server"]
