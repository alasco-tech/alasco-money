# Contributing

## Setup

- [Install mise](https://mise.jdx.dev/getting-started.html)
- `mise install`
- `mise run install`

## Development

- Run tests: `mise run test`

## Releasing

- Update version: `cargo bump patch` (or `minor`/`major`)
- Tag commit with `v${version}`
- CI handles the rest
