FROM rust:1.89-bookworm as builder

WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

# FROM debian:bookworm-slim
# RUN apt-get update 
# RUN apt-get upgrade -y
# RUN apt-get install -y openssl pkg-config libssl-dev
# RUN rm -rf /var/lib/apt/lists/*
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/video-storage /usr/local/bin/video-storage
CMD ["video-storage"]
