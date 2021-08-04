FROM rustlang/rust:nightly
WORKDIR /app/backend
COPY . .
EXPOSE 8000

# TODO: see if there's a way to add frontend compilation here
# Testing
RUN cargo build
CMD ["cargo", "run"]

# # Production
# RUN ROCKET_ENV=prod cargo build --release
# CMD ["cargo", "run", "--release"]
