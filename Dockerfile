FROM rust:1.22.1-stretch

WORKDIR /usr/src/users
COPY . .

RUN cargo install

CMD ["users"]
