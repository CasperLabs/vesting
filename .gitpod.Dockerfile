
FROM gitpod/workspace-full

# Install custom tools, runtimes, etc.
# For example "bastet", a command-line tetris clone:
# RUN brew install bastet
#
# More information: https://www.gitpod.io/docs/config-docker/
RUN bash -cl "rustup component add cargo --toolchain nightly-2020-07-05-x86_64-unknown-linux-gnu"
RUN sudo apt-get -y install \
              protobuf-compiler libprotobuf-dev 
RUN sudo apt clean

