// wordCount example

import { context } from "./stream/context";
import { kafka, redis } from "./connectors/connectors";
import ExecutionContext = context.ExecutionContext;
import Kafka = kafka.Kafka;
import Redis = redis.Redis;

async function wordCount(ctx: ExecutionContext) {
  let source = Kafka
    .builder()
    .brokers(["localhost:9092"])
    .topic("topic")
    .group("word_count")
    .build<string>(undefined, typeof "");


  let sink = Redis.new<{ t0: number, t1: string }>()
    .host("localhost")
    .keyExtractor((v) => v.t1)
    .valueExtractor((v) => v.t0.toString());

  let stream = source.createFlow(ctx);

  await stream.flatMap(value => value.split(" ").map(v => {
    return { t0: 1, t1: v };
  }))
    .keyBy(v => v.t1)
    .window({
      fixed: {
        size: {
          seconds: 3
        }
      }
    })
    .reduce((v1, v2) => {
      return { t1: v1.t1, t0: v1.t0 + v2.t0 };
    })
    .sink(sink)
    .execute();
}

wordCount(ExecutionContext.new("wordCount", "default")).then();