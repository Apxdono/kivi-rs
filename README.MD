# kivi-rs

**K**i**V**i is a Rust based CLI tool to browse and interract with various **K**ey-**V**alue storages.

## MVP

For MVP release will try to provide support of following KV Storages:
- `Consul` w/ token auth
- `Vault` w/ token auth
- `etcd`

## Expansion plans

In observable future might extend the reach to `Zookeper`, `Eureka` and `SurrealDB`.

## Inspiration

My pet project to learn Rust.

Crappy code expected!

**Use at own risk**.

## Building

Requires [Rust Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) to be installed.

```sh
cargo build
```


## Running locally

```sh
cargo run <arguments>

# OR
cargo build

./target/debug/kivi -h
```

Additionally you can spin up KV storages in Docker using `docker-compose`

```sh
docker-compose up -d
```
### Setup Consul

Once Consul Docker container is up and running you can enable ACL subsystem,
populate values, place token in .consul-token file by running:
```sh
./consulboot.sh
```

Consul: [local UI link](http://127.0.0.1:8500/ui/)
