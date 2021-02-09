FROM alpine:latest

WORKDIR /usr/src/app

RUN apt update -qq && apt upgrade -qq

RUN wget https://get.docker.com -O get-docker.sh
RUN sh get-docker.sh
RUN rm -rf /var/lib/apt/lists/*

COPY docker2mqtt .

ENTRYPOINT ["/usr/src/app/docker2mqtt"]