<div align="center">
  <h1>Lightflus</h1>
  <p>
    <strong>Lightflus is a cloud-native distributed stateful dataflow engine. </strong>
  </p>
  <p>

[![CI](https://github.com/Lady-Summer/lightflus-runtime/actions/workflows/workflow.yml/badge.svg)](https://github.com/Lady-Summer/lightflus-runtime/actions/workflows/workflow.yml) [![Join the chat at https://gitter.im/lightflus/community](https://badges.gitter.im/lightflus/community.svg)](https://gitter.im/lightflus/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
</p>
</div>

**Important Notes**

1. Lightflus is in **preview** now, some features are still in developing. If you want to try a better Lightflus, please wait for the demo version released.

2. We will be very appreciate if you can give us any feedback, including design, features support and more;


## Scenarios for Lightflus

1. Large-scale real-time computation;
2. CDC (Change Data Capture);
3. Data Integration Pipeline;



## Design Philosophy

### Target Users

Lightflus is **designed for most developer teams even no one is familiar with streaming computation**. Any of your team member can write a dataflow task and deploy it on production. Lightflus can connect with any event source (Kafka, MQTT, etc) in your cloud infra and emit the output result into the storage sink (mysql, redis, etc) which is processed by user-defined Dataflow. 


### Typescript API + Rust Runtime

Lightflus is powered by [Deno](https://github.com/denoland/deno) and implemented in Rust which can ensure memory safe and real-time performance. We embed `v8` engine into Lightflus engine with minimum dependencies makes it light and fast; With the help of `Deno`, you can **run `Typescript` user-defined functions or `WebAssembly` encoded bytes code (for better performance) in Lightflus with stream-like API**; 


## Document
You can read the [document](https://humorous-bream-e48.notion.site/Lightflus-Document-217eedc73610413ba2a4f0c374d66c77) for more details about Lightflus;

### Notes

The document of Lightflus is for the demo version. In preview version, some features are still *in developing*. Therefore, **we recommand you to try Lightflus after Demo released**;


## Roadmap

We use Jira to manage the development progress of Lightflus;

You can get the Roadmap in this [Jira Dashboard](https://lightflus.atlassian.net/jira/software/c/projects/LIG/boards/1/roadmap?shared=&atlOrigin=eyJpIjoiOWJhOTRiOGNkZTBlNDY5OWFkZWU4ZGQxYjRkYTg3MTkiLCJwIjoiaiJ9)

## Community

Welcome all to join [Slack](https://lightflus.slack.com/join/shared_invite/zt-1hqwryop3-jWOhWSuQ2B7wulhQM5~sHQ#/shared-invite/email) Community! 

## Set Up

If you want to start Lightflus on your computer, there are two ways:

1. Start by Binary Packages (**Not Recommanded**)

```shell
$ cargo install --path src/worker

$ cargo install --path src/coordinator
```

Then Run by command

```shell
$ target/release/worker

$ target/release/coordinator
```

2. Start by Docker Compose (**Recommended For Running Background**)

```shell
$ docker-compose up
```