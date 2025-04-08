FROM rust:1.86 as builder

WORKDIR /src/rtd

COPY . .

RUN cargo install --path . --root .


FROM debian:bookworm

RUN apt-get update && apt-get install -y cron openssl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/rtd/bin/ems-rtd-api /usr/local/bin/ems-rtd-api
COPY .env .
CMD ["ems-rtd-api"]
