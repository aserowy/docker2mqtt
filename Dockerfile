FROM python:latest

WORKDIR /usr/src/app

RUN apt-get update -qq && \
    apt-get upgrade -qq && \
    curl -fsSL https://get.docker.com -o get-docker.sh && \
    sh get-docker.sh

RUN pip install --upgrade pip
RUN pip install --no-cache-dir paho-mqtt

COPY docker2mqtt .

ENTRYPOINT ["/usr/src/app/docker2mqtt"]