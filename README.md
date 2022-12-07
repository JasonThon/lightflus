<div align="center">
  <h1>Lightflus</h1>
  <p>
    <strong>Lightflus is a cloud-native distributed stateful dataflow engine. </strong>
  </p>
  <p>

[![CI](https://github.com/Lady-Summer/lightflus-runtime/actions/workflows/workflow.yml/badge.svg)](https://github.com/Lady-Summer/lightflus-runtime/actions/workflows/workflow.yml) [![Join the chat at https://gitter.im/lightflus/community](https://badges.gitter.im/lightflus/community.svg)](https://gitter.im/lightflus/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![codecov](https://codecov.io/gh/JasonThon/lightflus/branch/master/graph/badge.svg?token=7Y1MMDWNG5)](https://codecov.io/gh/JasonThon/lightflus)
</p>
</div>

**Important Notes**

1. Lightflus has released the **demo version**. Welcome to try it! 

2. Your feedback is very important and we will take very serious to any of your advice!


## Scenarios for Lightflus

1. Large-scale real-time computation;
2. CDC (Change Data Capture);
3. Real-Time Data Pipeline Integration;


## Design Philosophy

### Target Users

Lightflus is **designed for most developer teams even no one is familiar with streaming computation**. Any of your team member can write a dataflow task and deploy it on production. Lightflus can connect with any event source (Kafka, MQTT, etc) in your cloud infra and emit the output result into the storage sink (mysql, redis, etc) which is processed by user-defined Dataflow. 


### Typescript API + Rust Runtime

Lightflus is powered by [Deno](https://github.com/denoland/deno) and implemented in Rust which can ensure memory safe and real-time performance. We embed `v8` engine into Lightflus engine with minimum dependencies makes it light and fast; With the help of `Deno`, you can **run `Typescript` user-defined functions or `WebAssembly` encoded bytes code (for better performance) in Lightflus with stream-like API**; 

### References

Lightflus is mainly based on Google's Paper [The Dataflow Model: A Practical Approach to Balancing Correctness, Latency, and Cost in Massive-Scale, Unbounded, Out-of-Order Data Processing](https://research.google/pubs/pub43864/) and refer to [Streaming System](https://www.oreilly.com/library/view/streaming-systems/9781491983867/). Some other papers in the field of streaming system are also our important source of references. 


## Document
You can read the [document](https://humorous-bream-e48.notion.site/Lightflus-Document-217eedc73610413ba2a4f0c374d66c77) for more details about Lightflus;

## Roadmap

We use Jira to manage the development progress of Lightflus;

You can get the Roadmap in this [Jira Dashboard](https://lightflus.atlassian.net/jira/software/c/projects/LIG/boards/1/roadmap?shared=&atlOrigin=eyJpIjoiOWJhOTRiOGNkZTBlNDY5OWFkZWU4ZGQxYjRkYTg3MTkiLCJwIjoiaiJ9)

## Community

You can join [Gitter](https://gitter.im/lightflus/community) community!


## Set Up Lightflus

### Start from Source

```bash
$ cargo run --manifest-path src/worker/Cargo.toml
$ cargo run --manifest-path src/coordinator/Cargo.toml
$ cargo run --manifest-path src/apiserver/Cargo.toml
```

### Start by Docker (**Recommended**)

```bash
$ docker-compose up
```

## Try Lightflus

### Preparation

1. Install Node.JS environment
2. Use WebStorm / VS Code to create a new Node.JS project
3. intialize typescript project
   1. install typescript dependency: `npm install typescript`
   2. initialize `tsconfig.json`: 
    ```bash 
    yarn tsc -p .
    ```
4. install `lightflus-api` dependency: 
   ```bash 
   npm i lightflus-api
   ```

### Deploy Your first Lightflus Task
We use `word count` as the example to show you how to deploy a Lightflus dataflow task

1. Modify `tsconfig.json`

We recommand you to set up properties in `tsconfig.json` file like below:

```json
{
  "compilerOptions": {
    "module": "commonjs",
    "target": "es2016",
    "sourceMap": true,
    "baseUrl": "./",
    "incremental": true,
    "skipLibCheck": true,
    "strictNullChecks": false,
    "forceConsistentCasingInFileNames": false,
    "strictPropertyInitialization": false,
    "esModuleInterop": true,
    "moduleResolution": "node"
  }
}

```

2. Implement Word Count

```typescript
// wordCount example

import {context} from "lightflus-api/src/stream/context";
import {kafka, redis} from "lightflus-api/src/connectors/connectors";
import ExecutionContext = context.ExecutionContext;
import Kafka = kafka.Kafka;
import Redis = redis.Redis;

async function wordCount(ctx: ExecutionContext) {
    // fetch string stream from kafka
    let source = Kafka
        .builder()
        .brokers(["kafka:9092"])
        // 消费的 topic 为 topic
        .topic("topic_2")
        // groupId 为 word_count
        .group("word_count")
        // 反序列化的类型
        .build<string>(undefined, typeof "");

    // It will persist the counting values in Redis
    let sink = Redis.new<{ t0: number, t1: string }>()
        .host("redis")
        .keyExtractor((v) => v.t1)
        .valueExtractor((v) => v.t0.toString());

    // create a Dataflow
    let stream = source.createFlow(ctx);

    // We design the Dataflow
    await stream.flatMap(value => value.split(" ").map(v => {
        return {t0: 1, t1: v};
    }))
        .keyBy(v => v.t1)
        .reduce((v1, v2) => {
            return {t1: v1.t1, t0: v1.t0 + v2.t0};
        })
        // write the results into Redis sink
        .sink(sink)
        // Then execute
        .execute();
}

wordCount(ExecutionContext.new("wordCount", "default")).then();
```

3. Compile typescript codes

```bash
$ yarn tsc -p .
```

4. Set environment variables

```bash
$ export LIGHTFLUS_ENDPOINT=localhost:8080
```

5. Run Javascript code after compilation

```bash
$ node wordCount.js
```

### Make the Dataflow Run!

You can send message to Kafka

```text
hello hello hello world world world
```

And you can get values in Redis

```bash
redis> GET hello
"3"

redis> GET world
"3"
```

# Contribution
Please read [CONTRIBUTING.md](CONTRIBUTING.md) document
