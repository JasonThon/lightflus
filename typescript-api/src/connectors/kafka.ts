import { Source } from "./definition";
import { common } from "../proto/apiserver";
import ISource = common.ISource;
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
    return kafka;
  }
}


