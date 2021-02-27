FROM debian:buster-slim

RUN apt update && \
apt install --no-install-recommends -y \
apt-transport-https \
ca-certificates \
curl \
gnupg \
gnupg-agent \
software-properties-common

# -k workaround on armhf envs
RUN curl -fsSL -k https://download.docker.com/linux/debian/gpg | apt-key add - && \
add-apt-repository \
"deb [arch=armhf] https://download.docker.com/linux/debian \
    $(lsb_release -cs) \
    stable"

RUN apt-get update && \
apt-get install --no-install-recommends -y docker-ce-cli

RUN rm -rf /var/lib/apt/lists/*

# configuration and persistance of docker2mqtt
RUN mkdir -p /docker2mqtt/config
RUN mkdir -p /docker2mqtt/logs

VOLUME ["/docker2mqtt/logs", "/docker2mqtt/config"]

COPY /target/armv7-unknown-linux-gnueabihf/release/docker2mqtt /docker2mqtt/

ENTRYPOINT ["/docker2mqtt/docker2mqtt"]
