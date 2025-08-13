FROM rust:1.89

WORKDIR /usr/src/app
RUN apt-get update -y
RUN apt-get upgrade -y
COPY . .
RUN ls -l
RUN cargo install --path . 

CMD ["video-streaming"]