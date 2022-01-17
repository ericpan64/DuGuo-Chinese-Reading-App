FROM rustlang/rust:nightly-slim
WORKDIR /app
COPY . .
RUN apt-get update &&\
    apt-get install -y\
    libssl-dev\
    pkg-config
RUN cargo clean
EXPOSE 8000

# Testing
RUN cargo build
CMD ["cargo", "run"]

# # Production
# RUN ROCKET_ENV=prod cargo build --release
# CMD ["cargo", "run", "--release"]