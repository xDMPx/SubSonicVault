FROM node:24-alpine AS player-build
WORKDIR /usr/src/myapp
COPY . .
WORKDIR /usr/src/myapp/player
RUN npm install
RUN npm run build

FROM rust:1.92.0 AS builder
WORKDIR /usr/src/myapp
COPY --from=player-build /usr/src/myapp /usr/src/myapp
RUN cargo install --path .

FROM debian:trixie-slim
WORKDIR /usr/src/myapp
COPY --from=builder /usr/src/myapp /usr/src/myapp
COPY --from=builder /usr/local/cargo/bin/subsonic_vault /usr/local/bin/subsonic_vault
RUN mkdir /vault

CMD ["subsonic_vault", "/vault"]
