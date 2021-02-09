FROM debian:stable
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

WORKDIR /usr/src/app

RUN apt update -qq
RUN apt upgrade -qq
RUN apt install --no-install-recommends -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg \
    gnupg-agent \
    software-properties-common
RUN curl -fsSL https://get.docker.com -o get-docker.sh
RUN sh get-docker.sh
RUN rm -rf /var/lib/apt/lists/*

COPY docker2mqtt .

ENTRYPOINT ["/usr/src/app/docker2mqtt"]