FROM rust:1.89-bookworm as builder

WORKDIR /usr/src/app
COPY . .
RUN cargo install --debug --path . 

FROM debian:bookworm-slim
RUN apt-get update 
RUN apt-get upgrade -y
RUN apt-get install -y openssl
# RUN rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/video-streaming /usr/local/bin/video-streaming
CMD ["video-streaming"]