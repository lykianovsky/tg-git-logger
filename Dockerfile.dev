FROM rust:latest

RUN apt-get update && apt-get install -y build-essential git curl \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-watch

WORKDIR /usr/src/app

COPY . .

CMD ["cargo", "watch", "-w", "src", "-x", "run", "--poll"]

