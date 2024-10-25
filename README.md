# alasco-money

`Money`, `MoneyWithVAT` and `MoneyWithVATRatio` implemented in Rust for performance.

## Contributing

- [Install Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- `uv sync`
- Play with the code
- Run tests: `maturin develop && pytest`

### Releasing a new version
 - Update the version in `Cargo.toml`
 - Tag the correponding `main` commit with `v${version}`
 - Wait for the release to be created by CI
