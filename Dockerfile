ARG SUB_DIR
ARG BASE_IMAGE
FROM $BASE_IMAGE

# configuration and persistance of docker2mqtt
RUN mkdir -p /docker2mqtt/config
RUN mkdir -p /docker2mqtt/logs
RUN mkdir -p /docker2mqtt/data

VOLUME ["/docker2mqtt/logs", "/docker2mqtt/config", "/docker2mqtt/data"]

COPY ./target/${SUB_DIR}release/docker2mqtt /docker2mqtt/

ENTRYPOINT ["/docker2mqtt/docker2mqtt"]
