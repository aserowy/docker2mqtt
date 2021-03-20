FROM alpinelinux/docker-cli:latest

# configuration and persistance of docker2mqtt
RUN mkdir -p /docker2mqtt/config
RUN mkdir -p /docker2mqtt/logs
RUN mkdir -p /docker2mqtt/db

VOLUME ["/docker2mqtt/logs", "/docker2mqtt/config", "/docker2mqtt/db"]

COPY /target/armv7-unknown-linux-musleabihf/release/docker2mqtt /docker2mqtt/

ENTRYPOINT ["/docker2mqtt/docker2mqtt"]
