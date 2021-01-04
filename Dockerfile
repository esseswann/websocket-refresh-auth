FROM rust:1.49 as builder

RUN USER=root cargo new --bin websocket-refresh-auth
WORKDIR /websocket-refresh-auth
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/websocket_refresh_auth*
RUN cargo build --release

EXPOSE 9001
FROM debian:buster-slim
ARG APP=/usr/src/app

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /websocket-refresh-auth/target/release/websocket-refresh-auth ${APP}/websocket-refresh-auth

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./websocket-refresh-auth"]