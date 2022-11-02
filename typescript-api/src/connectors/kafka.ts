import { Source } from "./definition";
import { common } from "../proto/apiserver";
import ISource = common.ISource;
import DataTypeEnum = common.DataTypeEnum;

export class Kafka<T> extends Source<T> {
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
        dataType: this.getDataType()
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

  brokers(brokers: string[]): KafkaBuilder {
    this._brokers = brokers;
    return this;
  }

  topic(topic: string): KafkaBuilder {
    this._topic = topic;
    return this;
  }

  build<T>(ctor?: new() => T, type?: string): Kafka<T> {
    let kafka = new Kafka<T>();
    if (type != null) {
      kafka.type = type;
    } else if (ctor != null) {
      kafka.type = typeof new ctor();
    }
    kafka.topic = this._topic;
    kafka.brokers = this._brokers;
    return kafka;
  }
}


