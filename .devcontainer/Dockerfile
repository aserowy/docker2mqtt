FROM mcr.microsoft.com/vscode/devcontainers/rust:latest

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive

# Install Docker CE CLI
RUN apt-get install -y apt-transport-https ca-certificates curl gnupg2 lsb-release
RUN curl -fsSL https://download.docker.com/linux/$(lsb_release -is | tr '[:upper:]' '[:lower:]')/gpg | apt-key add - 2>/dev/null && \
    echo "deb [arch=amd64] https://download.docker.com/linux/$(lsb_release -is | tr '[:upper:]' '[:lower:]') $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list && \
    apt-get update && \
    apt-get install -y docker-ce-cli

ARG NONROOT_USER=vscode

# Default to root only access to the Docker socket, set up non-root init script
RUN touch /var/run/docker-host.sock && \
    ln -s /var/run/docker-host.sock /var/run/docker.sock && \
    apt-get update && \
    apt-get -y install socat

# Create docker-init.sh to spin up socat
RUN echo "#!/bin/sh\n\
    sudoIf() { if [ \"\$(id -u)\" -ne 0 ]; then sudo \"\$@\"; else \"\$@\"; fi }\n\
    sudoIf rm -rf /var/run/docker.sock\n\
    ((sudoIf socat UNIX-LISTEN:/var/run/docker.sock,fork,mode=660,user=${NONROOT_USER} UNIX-CONNECT:/var/run/docker-host.sock) 2>&1 >> /tmp/vscr-docker-from-docker.log) & > /dev/null\n\
    \"\$@\"" >>/usr/local/share/docker-init.sh && \
    chmod +x /usr/local/share/docker-init.sh

# configuration and persistance of docker2mqtt
RUN sudo mkdir -p /docker2mqtt/config
RUN sudo mkdir -p /docker2mqtt/logs
RUN sudo mkdir -p /docker2mqtt/data

VOLUME [ "/docker2mqtt/config", "/docker2mqtt/logs" , "/docker2mqtt/data" ]

# VS Code overrides ENTRYPOINT and CMD when executing `docker run` by default.
# Setting the ENTRYPOINT to docker-init.sh will configure non-root access to
# the Docker socket if "overrideCommand": false is set in devcontainer.json.
# The script will also execute CMD if you need to alter startup behaviors.
ENTRYPOINT [ "/usr/local/share/docker-init.sh" ]
CMD [ "sleep", "infinity" ]
