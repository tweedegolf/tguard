# TGuard

TGuard is a web-based sending and decrypting service for [irmaseal](https://github.com/encryption4all/irmaseal)-encrypted messages that is currently in development at Tweede Golf. TGuard utilizes [IRMA](https://irma.app/) to allow an user to encrypt messages client-side. These messages can be decrypted client-side once the receiver proofs to be the owner of attributes the message was encrypted for, like an e-mail address, name or an identifying number.

[Screenshot of TGuard](screen.jpg)

## Running TGuard

Tguard supports local running through a docker setup. For this you need to have both docker and docker-compose installed. The application uses a database, which can be initialized with the `./setup.sh` script included. After this, a local copy of the application can be started `docker-compose up`, and the tguard website will be localy available at http://tguard.localhost

### Dependencies

The easyest way to start developing with this software is using docker-compose. The docker files contain all software neccecary to run the application (eg. Postgres, Nginx, Rust).

Currently we use the following software versions for this project:

- Rust version 1.57 (see [rust:bullseye](https://hub.docker.com/_/rust))
- NGINX version 1.21
- Postgres version 12
- Mailhog version 1.0

In addition to Rust the wasm target and the cargo packages `trunk` and `wasm-bindgen-cli` must be installed:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk wasm-bindgen-cli
```

### Technical overview

TGuard is written in Rust, both the back-end and the front-end. The front-end is compiled and bundled using [trunk](https://trunkrs.dev/) and uses the front-end framework [yew](https://yew.rs/).

The other Rust libraries used can be found in `Cargo.toml` in both the frontend and backend directories.

For a technical overview of [IRMA](https://irma.app/docs/what-is-irma/) you can consult this resource. IRMA Seal has a technical overview that can be found [here](https://github.com/Wassasin/irmaseal/blob/master/docs/design.md).

## Funding

This project was funded through the NGI0 PET Fund, a fund established by NLnet with financial support from the European Commission's Next Generation Internet programme.