import { Sink } from "./definition";
import { injectFunctionName } from "../common/common";
import { common } from "../proto/apiserver";
import ISink = common.ISink;
import IStatement = common.MysqlDesc.IStatement;
import Extractor = common.MysqlDesc.Statement.Extractor;
import IConnectionOpts = common.MysqlDesc.IConnectionOpts;
import ConnectionOpts = common.MysqlDesc.ConnectionOpts;

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