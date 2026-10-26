# Build the frontend and server, then ship a small runtime image.
FROM rust:1-bookworm AS build
RUN rustup target add wasm32-unknown-unknown && cargo install trunk --locked
WORKDIR /app
COPY . .
RUN cd web && trunk build --release
RUN cargo build -p codesprout-server --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /app/target/release/codesprout-server /app/codesprout-server
COPY --from=build /app/web/dist /app/web/dist
ENV ADDR=0.0.0.0:3000 DATABASE_PATH=/data/codesprout.db WEB_DIST=/app/web/dist
VOLUME /data
EXPOSE 3000
CMD ["/app/codesprout-server"]
