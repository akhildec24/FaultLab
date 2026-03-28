FROM ghcr.io/gleam-lang/gleam:latest

WORKDIR /app

COPY gleam.toml manifest.toml ./
RUN gleam deps

COPY . .

RUN gleam build

EXPOSE 4000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:4000/health || exit 1

CMD ["gleam", "run"]
