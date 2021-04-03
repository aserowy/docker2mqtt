FROM rust:alpine3.13 AS build

RUN mkdir /docker2mqtt

WORKDIR /docker2mqtt

RUN apk add --no-cache musl-dev

COPY ./ .

RUN cargo build --release

FROM alpinelinux/docker-cli:latest

# configuration and persistance of docker2mqtt
RUN mkdir -p /docker2mqtt/config
RUN mkdir -p /docker2mqtt/logs
RUN mkdir -p /docker2mqtt/data

VOLUME ["/docker2mqtt/logs", "/docker2mqtt/config", "/docker2mqtt/data"]

COPY --from=build /docker2mqtt/target/release/docker2mqtt /docker2mqtt/

ENTRYPOINT ["/docker2mqtt/docker2mqtt"]
