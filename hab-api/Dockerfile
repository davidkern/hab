FROM rust:1.69 as builder
WORKDIR /usr/src/hab-api
COPY . .
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/hab-api /usr/local/bin/hab-api

CMD ["hab-api"]
