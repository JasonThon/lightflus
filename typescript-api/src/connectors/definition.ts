import { ExecutionContext } from "../stream/context";
import { Dataflow } from "../stream/dataflow";
import { common } from "../proto/apiserver";
import { SourceOp } from "../stream/operator";
import ISource = common.ISource;
import ISink = common.ISink;

export abstract class Source<T> {
  createFlow(ctx: ExecutionContext): Dataflow<T> {
    ctx.addOperator(new SourceOp(ctx.getId(), this.asISource()).toOperatorInfo());
    return new Dataflow<T>(ctx);
  }

  abstract asISource(): ISource
}

export abstract class Sink<T> {

  abstract asISink(): ISink
}