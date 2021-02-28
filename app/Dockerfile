FROM rustlang/rust:nightly
WORKDIR /app
COPY . .
EXPOSE 8000

# Testing
RUN cargo build
CMD ["cargo", "run"]

# # Production
# RUN ROCKET_ENV=prod cargo build --release
# CMD ["cargo", "run", "--release"]