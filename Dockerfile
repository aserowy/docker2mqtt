FROM debian:buster-slim

RUN apt update
RUN apt install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg-agent \
    software-properties-common

# -k workaround on armhf envs
RUN curl -fsSL -k https://download.docker.com/linux/debian/gpg | apt-key add -
RUN add-apt-repository \
   "deb [arch=armhf] https://download.docker.com/linux/debian \
   $(lsb_release -cs) \
   stable"

RUN apt-get update
RUN apt-get install -y \
    docker-ce-cli \
    python3-paho-mqtt

RUN rm -rf /var/lib/apt/lists/*

COPY docker2mqtt .

ENTRYPOINT ["/docker2mqtt"]