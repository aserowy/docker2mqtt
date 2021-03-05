# docker2mqtt

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
      - ~/docker2mqtt/logs:/docker2mqtt/logs
      - /var/run/docker.sock:/var/run/docker-host.sock
```

In the directory `~/docker2mqtt/config` the configuration of the service is then done by `configuration.yaml`. Commented values are optional and are filled by corresponding defaults:

```yaml
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
  # client_id is the id to unquily identify the sender
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
