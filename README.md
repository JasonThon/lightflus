# Tableflow-rs

## Preparation

1. First and most: Install Rust environment:
    1. Go to official page: https://www.rust-lang.org/ to download installation pkg
2. CLion + Rust Plugin;
3. Tilt env

## Start up

### Installation

Run following commands at project root folder

1. Install binary packages

```shell
$ cargo install --path src/worker

$ cargo install --path src/connector

$ cargo install --path src/coordinator
```

2. Start Services

```shell
$ docker-compose up
```

## For Contribution

1. For now, we use single-branch development model. Every dev should develop on master branch.
2. For release branch, only admin has authority to initiate pull request. Then he/she will wait for all relevant dev to
   affirm;
3. Deployment will be triggered automatically after a release tag is published;

### How to release New Tag

**Tag's name template**:

* For test (master branch): ``dev.v{yyyyMMdd}.{version}``
* For production (release branch): ``prod.v{yyyyMMdd}.{version}``