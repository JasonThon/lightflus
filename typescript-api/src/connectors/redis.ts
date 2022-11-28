import { Sink } from "./definition";
import { common } from "../proto/apiserver";
import { injectFunctionName } from "../common/common";
import ISink = common.ISink;

export class Redis<V> extends Sink<V> {
  private _host: string;
  private _keyExtractor: (val: V) => any;
  private _valueExtractor: (val: V) => any;
  private _tls: boolean;

  private _password: string;

  set password(value: string) {
    this._password = value;
  }

  static new<T>(): Redis<T> {
    return new Redis<T>();
  }

  tls(value: boolean): Redis<V> {
    this._tls = value;
    return this;
  }

  valueExtractor(value: (val: V) => any): Redis<V> {
    this._valueExtractor = value;
    return this;
  }

  keyExtractor(value: (val: V) => any): Redis<V> {
    this._keyExtractor = value;
    return this;
  }

  host(value: string): Redis<V> {
    this._host = value;
    return this;
  }

  asISink(): ISink {
    return {
      redis: {
        connectionOpts: {
          host: this._host,
          password: this._password,
          tls: this._tls
        },
        keyExtractor: {
          function: injectFunctionName("redis_extractor", this._keyExtractor.toString())
        },
        valueExtractor: {
          function: injectFunctionName("redis_extractor", this._valueExtractor.toString())
        }
      }
    };
  }
}