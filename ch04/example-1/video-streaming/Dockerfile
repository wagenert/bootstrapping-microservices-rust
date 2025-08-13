FROM ubuntu:24.04

WORKDIR /usr/src/app
RUN apt-get update -y
RUN apt-get upgrade -y
COPY ./target/release/video-streaming ./ 
CMD ["./video-streaming"]