import { AdjacentList, Node } from "../common/common";

export class DataflowRequest {
  meta: AdjacentList;
  operators: Map<number, object>;

  constructor() {
    this.meta = new Array<Node>();
    this.operators = new Map<number, object>();
  }

  toString(): string {
    let ops = {};
    this.operators.forEach((val, key) => ops[key] = val);
    return JSON.stringify({
      meta: this.meta,
      operators: ops
    });
  }
}