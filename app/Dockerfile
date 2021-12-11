FROM rustlang/rust:nightly-slim
WORKDIR /app
COPY . .
EXPOSE 8000
RUN apt-get update
RUN apt-get install -y\
    libssl-dev\
    pkg-config
RUN cargo clean

# Testing
RUN cargo build
CMD ["cargo", "run"]

# # Production
# RUN ROCKET_ENV=prod cargo build --release
# CMD ["cargo", "run", "--release"]