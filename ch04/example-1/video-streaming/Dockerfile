FROM ubuntu:24.04

WORKDIR /usr/src/app
COPY ./video ./video
RUN apt-get update -y
RUN apt-get upgrade -y
COPY ./target/aarch64-unknown-linux-gnu/release/my-new-project ./ 
CMD ["./my-new-project"]