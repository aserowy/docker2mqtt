FROM debian:stable
ENV LC_ALL=C.UTF-8
ENV LANG=C.UTF-8

WORKDIR /usr/src/app

RUN apt update -qq \
&& apt upgrade -qq \
&& apt install --no-install-recommends -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg \
    gnupg-agent \
    software-properties-common \
&& curl -fsSL https://get.docker.com -o get-docker.sh \
&& sh get-docker.sh \
&& rm -rf /var/lib/apt/lists/*

COPY docker2mqtt .

ENTRYPOINT ["/usr/src/app/docker2mqtt"]