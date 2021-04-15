# docker2mqtt

[![release][release-badge]][github-url]
[![ci][ci-badge]][ci-url]
[![license][mit-badge]][mit-url]
[![pulls][pulls-badge]][docker-url]
[![size][size-badge]][docker-url]

[github-url]: https://github.com/aserowy/docker2mqtt
[release-badge]: https://img.shields.io/github/v/release/aserowy/docker2mqtt?sort=semver
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/aserowy/docker2mqtt/blob/master/LICENSE
[ci-badge]: https://github.com/aserowy/docker2mqtt/actions/workflows/ci.yml/badge.svg?branch=main
[ci-url]: https://github.com/aserowy/docker2mqtt/actions/workflows/ci.yml
[pulls-badge]: https://img.shields.io/docker/pulls/serowy/docker2mqtt
[size-badge]: https://img.shields.io/docker/image-size/serowy/docker2mqtt
[docker-url]: https://hub.docker.com/r/serowy/docker2mqtt

## introduction

docker2mqtt enables monitoring of your docker containers via mqtt. In addition, docker2mqtt also supports Home Assistant discovery and creates a single device for each container, where different sensors can be used to monitor the current state of the container.

This implementation is implemented in Rust. This keeps the image size small and creates an environment for long runtimes. docker2mqtt relies on the docker.sock to read out current states.

## configuration

docker2mqtt is configured using yaml. The confiugation is then provided to the container via volumes. In a docker-compose.yaml, the container can be initialized as follows:

```yaml
version: "3.0"
services:
  docker2mqtt:
    image: serowy/docker2mqtt:latest
    container_name: docker2mqtt
    restart: always
    volumes:
      - ~/docker2mqtt/config:/docker2mqtt/config
      - ~/docker2mqtt/data:/docker2mqtt/data
      - ~/docker2mqtt/logs:/docker2mqtt/logs
      - /var/run/docker.sock:/var/run/docker.sock
```

In the directory `~/docker2mqtt/config` the configuration of the service is then done by `configuration.yaml`. Commented values are optional and are filled by corresponding defaults:

```yaml
# docker:
  # persist_state enables persistence of the current state to handle container changes while docker2mqtt
  #   is asleep. This ensures that e.g. home assistant sensors are up to date. [default: false]
  # persist_state: true

  # stream_logs enables streams for container logs with mqtt. [default: true]
  # stream_logs: false

  # stream_logs_container is a white list filter for container names/ids. [default: empty]
  # IMPORTANT: do not enable logging for e.g. mosquitto or hassio because it can cause feedback loops!
  # stream_logs_container:
  #   - docker2mqtt
  #   - watchtower
  #   - borg

  # stream_logs_filter is a case insensitive white list filter for streamed logs. [default: empty]
  # stream_logs_filter:
  #   - error
  #   - test_word
  #   - my name, surname

# hassio:
  # discovery enables (true) or disables (false) discovery messages for home assistant
  discovery: true

  # discovery_prefix should point to the configured prefix in home assistant [default: homeassistant]
  # discovery_prefix:

  # device_prefix is used to prefix all created devices (container) in home assistant [default: docker]
  # device_prefix:

# logging:
  # sets the logging level (TRACE, DEBUG, INFO, WARN, and ERROR) at start up [default: INFO]
  # level:

mqtt:
  # client_id is the id to uniquely identify the sender
  client_id:

  # host is the remote hostname of your mqtt broker e.g. mosquitto
  host:

  # port of your mqtt broker e.g. 1883 for mosquitto
  port:

  # password: # default: None
  # username: # default: None

  # connection_timeout: # default: 20
  # keep_alive: # default: 30
  # qos: # default: 0
```
