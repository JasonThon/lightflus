import { Dataflow } from "../stream/dataflow";
import { common } from "../proto/apiserver";
import { SourceOp } from "../stream/operator";
import { injectFunctionName } from "../common/common";
import { context } from "../stream/context";
import ISource = common.ISource;
import ISink = common.ISink;

export namespace connectors {

  import ExecutionContext = context.ExecutionContext;

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
}

export namespace kafka {

  import Source = connectors.Source;
  import DataTypeEnum = common.DataTypeEnum;

  export class Kafka<T> extends Source<T> {
    private _partition: number = 0;

    set partition(value: number) {
      this._partition = value;
    }

    private _group: string;

    set group(value: string) {
      this._group = value;
    }

    private _topic: string;

    set topic(value: string) {
      this._topic = value;
    }

    private _brokers: string[];

    set brokers(value: string[]) {
      this._brokers = value;
    }

    private _type: string;

    set type(value: string) {
      this._type = value;
    }

    static builder(): KafkaBuilder {
      return new KafkaBuilder();
    }

    asISource(): ISource {
      return {
        kafka: {
          topic: this._topic,
          brokers: this._brokers,
          dataType: this.getDataType(),
          opts: {
            partition: this._partition,
            group: this._group
          }
        }
      };
    }

    private getDataType(): DataTypeEnum {
      if (this._type == "string") {
        return DataTypeEnum.DATA_TYPE_ENUM_STRING;
      }
      if (this._type == "object") {
        return DataTypeEnum.DATA_TYPE_ENUM_OBJECT;
      }
      if (this._type == "number") {
        return DataTypeEnum.DATA_TYPE_ENUM_NUMBER;
      }
      if (this._type == "undefined") {
        return DataTypeEnum.DATA_TYPE_ENUM_NULL;
      }
      if (this._type == "null") {
        return DataTypeEnum.DATA_TYPE_ENUM_NULL;
      }
      if (this._type == "bigint") {
        return DataTypeEnum.DATA_TYPE_ENUM_BIGINT;
      }
      if (this._type == "boolean") {
        return DataTypeEnum.DATA_TYPE_ENUM_BOOLEAN;
      }

      return DataTypeEnum.DATA_TYPE_ENUM_UNSPECIFIED;
    }
  }

  export class KafkaBuilder {
    private _brokers: string[];
    private _topic: string;
    private _group: string;
    private _partition: number = 0;

    brokers(brokers: string[]): KafkaBuilder {
      this._brokers = brokers;
      return this;
    }

    topic(topic: string): KafkaBuilder {
      this._topic = topic;
      return this;
    }

    group(group: string): KafkaBuilder {
      this._group = group;
      return this;
    }

    partition(partition: number): KafkaBuilder {
      this._partition = partition;
      return this;
    }

    build<T>(ctor?: new() => T, type?: string): Kafka<T> {
      let kafka = new Kafka<T>();
      if (ctor != null) {
        kafka.type = typeof new ctor();
      } else if (type != null) {
        kafka.type = type;
      } else {
        throw "undefined kafka type";
      }

      kafka.topic = this._topic;
      kafka.brokers = this._brokers;
      kafka.partition = this._partition;
      kafka.group = this._group;
      return kafka;
    }
  }
}

export namespace mysql {
  import IConnectionOpts = common.MysqlDesc.IConnectionOpts;
  import ConnectionOpts = common.MysqlDesc.ConnectionOpts;
  import Extractor = common.MysqlDesc.Statement.Extractor;
  import IStatement = common.MysqlDesc.IStatement;
  import Sink = connectors.Sink;

  export class Mysql<InputT> extends Sink<InputT> {
    private _statement: Statement<InputT>;
    private _connection_opt: IConnectionOpts;

    static new<T>(): Mysql<T> {
      return new Mysql<T>();
    }

    statement(statement: string): Mysql<InputT> {
      this._statement = Statement.new<InputT>(statement);
      return this;
    }

    extractors(extractor: (statement: Statement<InputT>) => void): Mysql<InputT> {
      extractor(this._statement);
      return this;
    }

    connection(options: IConnectionOpts): Mysql<InputT> {
      this._connection_opt = options;
      return this;
    }

    asISink(): ISink {
      return {
        mysql: {
          statement: this._statement.asIStatement(),
          connectionOpts: this._connection_opt
        }
      };
    }
  }

  export class Statement<T> {
    private _statement: string;
    private _string_extractors: Map<number, (val: T) => string>;
    private _number_extractors: Map<number, (val: T) => number>;

    static new<T0>(statement: string): Statement<T0> {
      let s = new Statement<T0>();
      s._statement = statement;
      s._string_extractors = new Map<number, (val: T0) => string>();
      s._number_extractors = new Map<number, <T0>(val: T0) => number>();
      return s;
    }

    setString(index: number, extractor: (val: T) => string) {
      this._string_extractors.set(index, extractor);
    }

    setNumber(index: number, extractor: (v: T) => number) {
      this._number_extractors.set(index, extractor);
    }

    asIStatement(): IStatement {
      let extractors = Array.from(this._string_extractors.entries()).map((entry) => {
        return new Extractor({
          index: entry[0],
          extractor: injectFunctionName("mysql_extractor", entry[1].toString())
        });
      });

      let numberExtractors = Array.from(this._number_extractors.entries()).map((entry) => {
        return new Extractor({
          index: entry[0],
          extractor: injectFunctionName("mysql_extractor", entry[1].toString())
        });
      });

      extractors.push(...numberExtractors);

      return {
        statement: this._statement,
        extractors: extractors
      };
    }
  }

  export class MysqlConnectionOptionsBuilder {
    private _host: string;
    private _username: string;
    private _password: string;
    private _db: string;

    host(host: string): MysqlConnectionOptionsBuilder {
      this._host = host;
      return this;
    }

    username(username: string): MysqlConnectionOptionsBuilder {
      this._username = username;
      return this;
    }

    password(password: string): MysqlConnectionOptionsBuilder {
      this._password = password;
      return this;
    }

    database(db: string): MysqlConnectionOptionsBuilder {
      this._db = db;
      return this;
    }

    build(): IConnectionOpts {
      let options = new ConnectionOpts();
      options.host = this._host;
      options.database = this._db;
      options.password = this._password;
      options.username = this._username;
      return options;
    }
  }
}

export namespace redis {

  import Sink = connectors.Sink;

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

}