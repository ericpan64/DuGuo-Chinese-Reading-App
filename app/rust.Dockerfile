FROM rustlang/rust:nightly

# Libraries if using alpine
# RUN apk update
# RUN apk --no-cache --update-cache add gcc build-base

COPY ./app .
EXPOSE 8000

# Testing
RUN cargo build
CMD ["cargo", "run"]

# Production
# RUN ROCKET_ENV=prod cargo build --release
# CMD ["cargo", "run", "--release"]