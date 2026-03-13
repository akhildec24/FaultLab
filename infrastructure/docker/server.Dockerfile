FROM ghcr.io/gleam-lang/gleam:latest

WORKDIR /app

COPY gleam.toml manifest.toml ./
RUN gleam deps

COPY . .

EXPOSE 8080

CMD ["gleam", "run"]
