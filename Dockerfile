FROM debian:stable

RUN apt update && \
    apt install --no-install-recommends -y apt-transport-https ca-certificates curl gnupg gnupg-agent software-properties-common && \
    curl -fsSL https://download.docker.com/linux/debian/gpg | apt-key add - && \
    add-apt-repository "deb [arch=armhf] https://download.docker.com/linux/debian $(lsb_release -cs) stable" && \
    apt update && \
    apt install --no-install-recommends -y docker-ce-cli python3-paho-mqtt && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY docker2mqtt .

ENTRYPOINT ["/usr/src/app/docker2mqtt"]