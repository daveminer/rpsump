FROM rust

WORKDIR /usr/src/rpsump

COPY . .

RUN apt-get update && \
   apt-get install -y protobuf-compiler

RUN rustup target add armv7-unknown-linux-gnueabihf

RUN cargo install cross

RUN curl -fsSL https://get.docker.com -o get-docker.sh && sh get-docker.sh

RUN systemctl start docker

RUN cross build --release --target=armv7-unknown-linux-gnueabihf

ENTRYPOINT "/bin/bash"
