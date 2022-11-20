import { ExecutionContext } from "./context";
import { Filter, FlatMap, KeyBy, MapOp, Reduce, SinkOp } from "./operator";
import { apiserver, common } from "../proto/apiserver";
import { Sink } from "../connectors/definition";
import { ApplicationStream, createResourceApiEndpoint } from "../common/consts";
import axios from "axios";
import IWindow = common.IWindow;
import CreateResourceRequest = apiserver.CreateResourceRequest;
import ResourceTypeEnum = apiserver.ResourceTypeEnum;

export class Dataflow<T> {
  protected ctx: ExecutionContext;
  protected readonly operator_id: number;

  constructor(ctx: ExecutionContext) {
    this.ctx = ctx;
    this.operator_id = ctx.getId();
  }

  flatMap<U>(callbackFn: (value: T) => U[]): Dataflow<U> {
    this.ctx.addChild(this.operator_id, new FlatMap<T, U>(this.ctx.incrementAndGetId(), callbackFn).toOperatorInfo());
    return new Dataflow<U>(this.ctx);
  }

  keyBy<U>(callbackFn: (value: T) => U): KeyedDataflow<U, T> {
    this.ctx.addChild(this.operator_id, new KeyBy<T, U>(this.ctx.incrementAndGetId(), callbackFn).toOperatorInfo());
    return new KeyedDataflow<U, T>(this.ctx);
  }

  filter(callbackFn: (value: T) => boolean): Dataflow<T> {
    this.ctx.addChild(this.operator_id, new Filter<T>(this.ctx.incrementAndGetId(), callbackFn).toOperatorInfo());
    return new Dataflow<T>(this.ctx);
  }

  sink(sink: Sink<T>): Dataflow<T> {
    this.ctx.addChild(this.operator_id, new SinkOp(this.ctx.incrementAndGetId(), sink.asISink()).toOperatorInfo());
    return new Dataflow<T>(this.ctx);
  }

  async execute() {
    let options = this.ctx.getCreateDataflowOptions();
    let request = new CreateResourceRequest();
    request.dataflow = options;
    request.resourceType = ResourceTypeEnum.RESOURCE_TYPE_ENUM_DATAFLOW;
    request.namespace = this.ctx.namespace;
    await axios.post(
      createResourceApiEndpoint,
      CreateResourceRequest.encode(request).finish(),
      { headers: { "Content-Type": ApplicationStream } })
      .then((resp) => {
        console.log(`{status: ${resp.status}, response: ${JSON.stringify(resp.data)}`);
      }).catch((err) => {
        if (axios.isAxiosError(err)) {
          console.log(`{status: ${err.status}, errMsg: ${JSON.stringify(err.response.data)}`);
        } else {
          console.log(err);
        }
      });
  }

  map<U>(callbackFn: (value: T) => U): Dataflow<U> {
    this.ctx.addChild(this.operator_id, new MapOp<T, U>(this.ctx.incrementAndGetId(), callbackFn).toOperatorInfo());
    return new Dataflow<U>(this.ctx);
  }

}

export class KeyedDataflow<K, T> extends Dataflow<T> {
  constructor(ctx: ExecutionContext) {
    super(ctx);
  }

  reduce(callbackFn: (agg: T, current: T) => T): KeyedDataflow<K, T> {
    this.ctx.addChild(this.operator_id, new Reduce(this.ctx.incrementAndGetId(), callbackFn).toOperatorInfo());
    return new KeyedDataflow<K, T>(this.ctx);
  }

  window(window: IWindow): KeyedDataflow<K, T> {
    this.ctx.setWindow(this.operator_id, window);
    return this;
  }

  filter(callbackFn: (value: T) => boolean): Dataflow<T> {
    super.filter(callbackFn);
    return new KeyedDataflow<K, T>(this.ctx);
  }
}