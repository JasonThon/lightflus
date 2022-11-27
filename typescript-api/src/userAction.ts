import { ExecutionContext } from "./stream/context";
import { Kafka } from "./connectors/kafka";
import { Redis } from "./connectors/redis";

class UserAction {
  userId: string;
  itemId: string;
  action: number;
  timestamp: string;
}


function userAction(ctx: ExecutionContext) {
  let source = Kafka
    .builder()
    .brokers(["localhost:9092"])
    .topic("user_action_topic")
    .group("user_action")
    .build<UserAction>(UserAction);

  let sink = Redis.new<{ userId: string, weights: [{ factor: number, action: number, timestamp: string, itemId: string }] }>()
    .host("localhost")
    .keyExtractor((v) => v.userId)
    .valueExtractor((v) => v.weights);

  let dataflow = source.createFlow(ctx);
  dataflow.keyBy(action => {
    return action.userId;
  }).map(action => {
    enum Action {
      Like,
      Bookmark,
      Notice,
      Shared,
      Unlike
    }

    let weightFactor = 0;
    if (action.action == Action.Like) {
      weightFactor = 1;
    } else if (action.action == Action.Bookmark) {
      weightFactor = 2;
    } else if (action.action == Action.Notice) {
      weightFactor = 3;
    } else if (action.action == Action.Shared) {
      weightFactor = 4;
    }

    return {
      userId: action.userId,
      weights: [{ factor: weightFactor, action: action.action, timestamp: action.timestamp, itemId: action.itemId }]
    };
  }).reduce((agg, value) => {
    agg.weights.push(...value.weights);
    return agg;
  }).sink(sink).execute().then();
}

userAction(ExecutionContext.new("userAction", "default"));