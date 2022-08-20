import {DuplicatedKeyCode, StreamDeployFailed, TableflowException} from "../exception";
import {Sink} from "../sink/sink";
import {ApplicationJsonHeader, POST, StatusCreated, StreamGraphCreate, TableCreateUrl} from "../constants";
import {FieldStream} from "../stream/stream";
import fetch from "node-fetch"
import * as console from 'console';

export class Field implements Sink {
    name: string
    type: DataType

    constructor(name: string, type: DataType) {
        this.name = name;
        this.type = type;
    }

    private _tableName: string

    public set tableName(value: string) {
        this._tableName = value;
    }

    public stream(): FieldStream {
        let stream = new FieldStream();
        stream.fieldName = this.name
        stream.tableName = this._tableName
        return stream
    }

    sink(graph: string): void {

        fetch(StreamGraphCreate, {
            headers: ApplicationJsonHeader,
            method: POST
        }).then(r => {
            if (r.status == StatusCreated) {
                console.log("create stream graph success")
            } else {
                throw new TableflowException(StreamDeployFailed, "stream graph deployed failed")
            }
        })
    }
}

export class Table {
    private readonly _name: string
    private _fields: Field[]

    constructor(name: string) {
        this._name = name
    }

    public static new(name: string): Table {
        return new Table(name)
    }

    public addField(name: string, type: DataType): Table {
        for (let i = 0; i < this._fields.length; i++) {
            if (this._fields[i].name == name) {
                throw new TableflowException(DuplicatedKeyCode, `Duplicated Key ${name}`)
            }
        }

        this._fields.push(new Field(name, type))
        return this
    }

    /**
     * create() will create a new table in current workspace
     */
    public create() {
        fetch(TableCreateUrl, {
            method: POST,
            body: JSON.stringify(this),
            headers: ApplicationJsonHeader
        }).then(r => {
            if (r.status == StatusCreated) {
                console.log("create table success")
            } else {

            }
        }).catch(err => console.error(err))
    }

    public field(name: string): Field {
        let field = new Field(name, DataType.UNSPECIFIC);
        field.tableName = this._name
        return field
    }
}

export enum DataType {
    UNSPECIFIC,
    INT,
    LONG,
    STRING,
    FLOAT,
    DOUBLE,
}