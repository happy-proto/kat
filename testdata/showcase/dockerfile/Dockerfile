# syntax=docker/dockerfile:1
FROM alpine:3.21 AS base

WORKDIR /app
ENV APP_HOME=/app

RUN printf '%s\n' "$APP_HOME" --version
CMD printf '%s\n' "preview"
ENTRYPOINT printf '%s\n' "$@"
