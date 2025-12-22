FROM rust:1.92.0

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .
RUN mkdir /vault

CMD ["subsonic_vault", "/vault"]
