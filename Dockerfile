FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
COPY . /static-wiki
WORKDIR /static-wiki
RUN cargo build --release

FROM alpine
MAINTAINER longfangsong@icloud.com
COPY --from=builder /static-wiki/target/release/static-wiki /
WORKDIR /
ENTRYPOINT ["/static-wiki"]
