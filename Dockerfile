FROM rust:1.92.0

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .
RUN apt update
RUN apt install nodejs npm -y
WORKDIR /usr/src/myapp/player
RUN npm install
RUN npm run build
RUN mkdir /vault

WORKDIR /usr/src/myapp
CMD ["subsonic_vault", "/vault"]
