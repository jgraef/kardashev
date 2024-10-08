# Kardashev

Kardashev is a MMORPG browser game about interstellar colonization. It is currently **in development** and **not yet functional**.


## Installation

### Rust

Kardashev is programmed in Rust 🦀, so you'll need the cargo build system, to compile it. The easiest way to install it is with [rustup](https://rustup.rs/).

### Database

The Kardashev server uses a PostgreSQL database to store the game state. Install PostgreSQL from the [official website](https://www.postgresql.org/), or with the package manager of your distribution.

Then setup the database:

```sh
sudo -u postgres psql
create user kardashev with encrypted password 'DATABASE PASSWORD';
create database kardshev;
grant all privileges on database kardashev to kardashev;
```

### Import stars

Right now the game world is pretty empty. Fill it with some stars! The CLI tool currently only accepts stars in the format from the [HYG catalog](https://github.com/astronexus/HYG-Database/blob/main/hyg/CURRENT/hygdata_v41.csv). Once you got the CSV, run:

```sh
cargo run --bin kardashev-cli -- admin import-stars --closest 1000 hygdata_v41.csv
```

This will import the 1000 stars closest to the sun.

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
