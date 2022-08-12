# Tableflow-Runtime

## Preparation

1. First, install Rust environment:
    1. Go to official page: https://www.rust-lang.org/ to download installation pkg
2. Visual Studio Code + Rust Plugin (**Recommended**);
3. Docker engine and swarm mode opened (**Recommended**);

## Start up

### Installation

Three ways to start Runtime:

1. Binary Packages

```shell
$ cargo install --path src/worker

$ cargo install --path src/coordinator
```

Then Run by command

```shell
$ target/release/worker

$ target/release/coordinator
```

2. Docker Compose (**Recommended**)

```shell
$ docker-compose up
```

3. Idea (**Recommended for Debug**)

## For Contributor

Welcome to Tableflow Team. Our mission is to create an advanced, high performance and stable streaming system which is
based on Dataflow Model in the Cloud.

You can read following documents to know more about Tableflow. If you have no access, please contact with the admin.

1. You can read the [Developer Guide](https://www.notion.so/Developer-Guide-bb6579a980844cff9b2702dd107e4ff3)
   to get more details for contribution.
2. [Architecture Overview](https://www.notion.so/Architecture-Overview-be9b006c61884db58e40dbd00e00b77d) will help you
   to get familiar with Tableflow Architecture Design
3. [Product Design](https://www.notion.so/Product-Design-efa990263c4b4e80a677243efc95a2f2) will tell you more about why we
   decide to create a streaming system by ourselves, not on the base of open source like Flink.
4. [Design Doc](https://www.notion.so/Design-282e33dc26a0416f9b25d20d78fe69d3) has the information of the historical
   design details of each version.

### How to release New Tag

**Tag's name template**:

* For test (master branch): ``dev.v{yyyyMMdd}.{version}``
* For production (release branch): ``prod.v{yyyyMMdd}.{version}``