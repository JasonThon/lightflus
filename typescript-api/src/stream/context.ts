import { apiserver, common } from "../proto/apiserver";
import ICreateDataflowOptions = apiserver.ICreateDataflowOptions;
import CreateDataflowOptions = apiserver.CreateDataflowOptions;
import Dataflow = common.Dataflow;
import IDataflowMeta = common.IDataflowMeta;
import IOperatorInfo = common.IOperatorInfo;
import IWindow = common.IWindow;
import DataflowMeta = common.DataflowMeta;

export class ExecutionContext {
  name: string;
  namespace: string;
  private readonly adjacentList: IDataflowMeta[];
  private operatorInfo: Map<number, IOperatorInfo>;
  private _currentId: number;

  constructor() {
    this._currentId = 0;
    this.adjacentList = [];
    this.operatorInfo = new Map<number, IOperatorInfo>();
  }

  static new(name: string, namespace: string): ExecutionContext {
    let context = new ExecutionContext();
    context.name = name;
    context.namespace = namespace;
    return context;
  }

  incrementAndGetId() {
    this._currentId += 1;
    return this._currentId;
  }

  getId(): number {
    return this._currentId;
  }

  addChild<T extends IOperatorInfo>(parentId: number, op: T) {
    this.addOperator(op);
    this.adjacentList.forEach((node) => {
      if (node.center == parentId) {
        node.neighbors.push(op.operatorId);
      }
    });
  }

  setWindow(operator_id: number, window: IWindow) {
    let info = this.operatorInfo.get(operator_id);
    this.operatorInfo.set(operator_id, info);
  }

  getCreateDataflowOptions(): ICreateDataflowOptions {
    let options = new CreateDataflowOptions();
    let df = new Dataflow();
    df.meta = this.adjacentList;

    this.operatorInfo.forEach((val, key) => {
      df.nodes[key] = val;
    });
    options.dataflow = df;
    return options;
  }

  addOperator<T extends IOperatorInfo>(operator: T) {
    this.operatorInfo = this.operatorInfo.set(operator.operatorId, operator);
    if (this.adjacentList.filter((node) => node.center == operator.operatorId).length == 0) {
      this.adjacentList.push(new DataflowMeta({ center: operator.operatorId, neighbors: [] }));
    }
  }
}