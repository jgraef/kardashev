# Kardashev

Kardashev is a MMORPG browser game about interstellar colonization. It is currently **in development** and **not yet functional**.


## Installation

First you'll need Rust 🦀. The easiest way to install it is with [rustup](https://rustup.rs/).

Then you'll need to install [PostgreSQL](https://www.postgresql.org/), and create a database for Kardashev:

```sh
sudo -u postgres psql
create user kardashev with encrypted password 'DATABASE PASSWORD';
create database kardshev;
grant all privileges on database kardashev to kardashev;
```

## Usage

Kardashev comes with a CLI tool to build assets and UI, and serve the API. It'll read some environment variables, so you don't have to pass everything via command line every time. Create the file `.env`:

```
# URL for the database we setup earlier.
DATABASE_URL="postgres://kardashev:DATABASE PASSWORD@localhost/kardashev"

# Configure logging
RUST_LOG=info,kardashev=debug

# Show backtraces when kardashev crashes
# RUST_BACKTRACE=1

# API URL for administrative commands
# KARDASHEV_API_URL="http://localhost:3000/api/v0"
```

To start a server with assets, UI and API, run:

```sh
cargo run --bin kardashev-cli -- serve --assets --ui
```

If you want to watch for changes in the assets or UI, and rebuild if necessary, add the `--watch` flag.
