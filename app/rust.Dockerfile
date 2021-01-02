FROM rustlang/rust:nightly-alpine
RUN apk update
RUN apk --no-cache --update-cache add gcc build-base
COPY ./app .
RUN ROCKET_ENV=prod cargo build --release
CMD ["cargo", "run", "--release"]