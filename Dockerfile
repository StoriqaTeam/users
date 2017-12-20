# Install Diesel CLI
FROM rust:1.22.1 as base
RUN cargo install diesel_cli

# Build app binary
FROM base as build
WORKDIR /app
COPY . .
RUN cargo install

# Run application
FROM build as release
CMD ["users"]