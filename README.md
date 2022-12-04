## About

REST server using the Rust Rocket and Diesel frameworks (using PostgreSQL).

Basic CRUD functionality to store events, pins, and faqs. 

Meant to be used with the "Corkboard" discord bot (still in development).

## Installation
Set up local Postgres server:
* https://www.postgresql.org/download/
* `sudo -u postgres psql postgres`
* `ALTER USER postgres WITH PASSWORD 'postgres';`
* `CREATE DATABASE corkboard WITH OWNER = postgres;`

Set up database DDL with diesel_cli:
* `cargo install diesel_cli --no-default-features --features postgres`
* `diesel migration run`

