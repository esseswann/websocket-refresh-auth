FROM rust:1.49 as builder

COPY ./ ./

RUN cargo build --release

RUN mkdir -p /build-out

RUN cp target/release/websocket_refresh_auth /build-out/
RUN cp ./src/index.html /build-out/

# Ubuntu 18.04
FROM ubuntu:18.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build-out/websocket_refresh_auth /
COPY --from=builder /build-out/index.html /

EXPOSE 9001
CMD /websocket_refresh_auth