import { injectFunctionName } from "../common/common";
import { common } from "../proto/apiserver";
import IWindow = common.IWindow;
import ISource = common.ISource;
import ISink = common.ISink;

export enum OperatorType {
  FlatMap = "flatMap",
  KeyBy = "keyBy",
  Reduce = "reduce",
  Source = "source",
  Sink = "sink",
  Map = "map",
  Filter = "filter",
  Window = "window"
}

export abstract class Operator {
  operatorId: number;
  window: IWindow;

  protected constructor(operatorId: number) {
    this.operatorId = operatorId;
  }

  toOperatorInfo(): common.IOperatorInfo {
    return {
      operatorId: this.operatorId
    };
  }

  abstract operatorType(): OperatorType;

  getFunctionName(): string {
    return `_operator_${this.operatorType()}_process`;
  }
}

export class FlatMap<T, U> extends Operator {
  private fn: (value: T) => U[];

  constructor(operatorId: number, fn: (value: T) => U[]) {
    super(operatorId);
    this.fn = fn;
  }

  operatorType(): OperatorType {
    return OperatorType.FlatMap;
  }

  toOperatorInfo(): common.IOperatorInfo {
    let info = super.toOperatorInfo();
    info.flatMap = {
      func: {
        function: injectFunctionName(this.getFunctionName(), this.fn.toString())
      }
    };
    return info;
  }
}

export class KeyBy<T, U> extends Operator {
  private fn: (value: T) => U;

  constructor(operatorId: number, fn: (value: T) => U) {
    super(operatorId);
    this.fn = fn;
  }

  operatorType(): OperatorType {
    return OperatorType.KeyBy;
  }

  toOperatorInfo(): common.IOperatorInfo {
    let info = super.toOperatorInfo();
    info.keyBy = {
      func: {
        function: injectFunctionName(this.getFunctionName(), this.fn.toString())
      }
    };
    return info;
  }
}

export class SinkOp<T> extends Operator {
  private readonly sink: ISink;

  constructor(operatorId: number, sink: ISink) {
    super(operatorId);
    this.sink = sink;
  }

  operatorType(): OperatorType {
    return OperatorType.Sink;
  }

  toOperatorInfo(): common.IOperatorInfo {
    let info = super.toOperatorInfo();
    info.sink = this.sink;
    return info;
  }
}

export class Reduce<T> extends Operator {
  private fn: (agg: T, current: T) => T;

  constructor(operatorId: number, fn: (agg: T, current: T) => T) {
    super(operatorId);
    this.fn = fn;
  }

  operatorType(): OperatorType {
    return OperatorType.Reduce;
  }

  toOperatorInfo(): common.IOperatorInfo {
    let info = super.toOperatorInfo();
    info.reducer = {
      func: {
        function: injectFunctionName(this.getFunctionName(), this.fn.toString())
      }
    };
    return info;
  }
}

export class SourceOp<T> extends Operator {
  private readonly source: ISource;

  constructor(operatorId: number, source: ISource) {
    super(operatorId);
    this.source = source;
  }

  operatorType(): OperatorType {
    return OperatorType.Source;
  }

  toOperatorInfo(): common.IOperatorInfo {
    let info = super.toOperatorInfo();
    info.source = this.source;
    return info;
  }

}

export class MapOp<T, U> extends Operator {
  private fn: (value: T) => U;

  constructor(operatorId: number, fn: (value: T) => U) {
    super(operatorId);
    this.fn = fn;
  }

  operatorType(): OperatorType {
    return OperatorType.Map;
  }

  toOperatorInfo(): common.IOperatorInfo {
    let info = super.toOperatorInfo();
    info.mapper = {
      func: {
        function: injectFunctionName(this.getFunctionName(), this.fn.toString())
      }
    };
    return info;
  }

}

export class Filter<T> extends Operator {
  private fn: (value: T) => boolean;

  constructor(operatorId: number, fn: (value: T) => boolean) {
    super(operatorId);
    this.fn = fn;
  }

  operatorType(): OperatorType {
    return OperatorType.Filter;
  }


  toOperatorInfo(): common.IOperatorInfo {
    let info = super.toOperatorInfo();
    info.filter = {
      func: {
        function: injectFunctionName(this.getFunctionName(), this.fn.toString())
      }
    };
    return info;
  }
}

export class WindowOp extends Operator {
  private readonly _window: IWindow;

  constructor(operatorId: number, window: IWindow) {
    super(operatorId);
    this._window = window;
  }

  operatorType(): OperatorType {
    return OperatorType.Window;
  }

  toOperatorInfo(): common.IOperatorInfo {
    this.check();
    let info = super.toOperatorInfo();
    info.window = this._window;
    return info;
  }

  getFunctionName(): string {
    return "";
  }

  private check() {
    if (this._window == null) {
      throw "window must not be null";
    }

    let isFixedNonNull = this._window.fixed != null;
    let isSlideNonNull = this._window.slide != null;
    let isSessionNonNull = this._window.session != null;

    if ((isFixedNonNull && isSlideNonNull) || (isFixedNonNull && isSessionNonNull) || (isSlideNonNull && isSessionNonNull)) {
      throw "You can only set window in one of [fixed, session, slide]";
    }

    if (!isSessionNonNull && !isFixedNonNull && !isSlideNonNull) {
      throw "You must set window from [fixed, session, slide]";
    }
  }
}