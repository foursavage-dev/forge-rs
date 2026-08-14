FROM rust:1.70 as rust-builder
WORKDIR /app
COPY rust-service/ .
RUN cargo build --release

FROM golang:1.21 as go-builder
WORKDIR /app
COPY go-service/ .
RUN go build -o go-service

FROM node:18 as ts-builder
WORKDIR /app
COPY web-frontend/ .
RUN npm install && npm run build

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=rust-builder /app/target/release/rust-service /usr/local/bin/
COPY --from=go-builder /app/go-service /usr/local/bin/
EXPOSE 8080 8081
CMD ["rust-service"]
