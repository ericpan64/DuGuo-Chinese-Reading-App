FROM rustlang/rust:nightly
WORKDIR /app
COPY . .
EXPOSE 8000

# TODO: add frontend compilation here
# Testing
RUN cargo build
CMD ["cargo", "run"]

# # Production
# RUN ROCKET_ENV=prod cargo build --release
# CMD ["cargo", "run", "--release"]
