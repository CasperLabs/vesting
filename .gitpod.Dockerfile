
FROM gitpod/workspace-full

# Install custom tools, runtimes, etc.
# For example "bastet", a command-line tetris clone:
# RUN brew install bastet
#
# More information: https://www.gitpod.io/docs/config-docker/
RUN sudo apt-get -y install \
              curl moreutils netcat-openbsd nmap openssh-server psmisc screen socat tmux wget \
              java-common jflex openjdk-11-jdk-headless openjdk-8-jdk-headless sbt=1.\* \
              protobuf-compiler libprotobuf-dev \
              python3.7 libpython3.7-dev python3-pip python3.8 libpython3.8-dev \
              docker-ce rpm fakeroot lintian nodejs rsync locales libssl-dev pkg-config jq

RUN sudo apt clean
