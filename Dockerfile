FROM node:24-alpine as player-build

WORKDIR /usr/src/myapp
COPY . .
WORKDIR /usr/src/myapp/player
RUN npm install
RUN npm run build

FROM rust:1.92.0

WORKDIR /usr/src/myapp
COPY --from=player-build /usr/src/myapp /usr/src/myapp

RUN cargo install --path .
RUN mkdir /vault

CMD ["subsonic_vault", "/vault"]
