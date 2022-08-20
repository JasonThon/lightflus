import {Connector, kafkaConnectorType} from "./conn";
import {ApplicationJsonHeader, ConnectorCreateUrl, POST} from "../constants";
import fetch from "node-fetch";

type KafkaConfig = {
    brokers: string[]
    topic: string
    partition: number
    credential: {
        username: string
        password: string
    }
    cgroup: string
    clientId: string
}

export class KafkaConnector extends Connector {
    private _topic: string
    private _clientId: string
    private _cgroup: string
    private _partition: number
    private _credential: Credential
    private _brokers: string[]

    brokers(value: string[]): KafkaConnector {
        this._brokers = value;
        return this
    }

    credential(value: Credential): KafkaConnector {
        this._credential = value;
        return this
    }

    partition(value: number): KafkaConnector {
        this._partition = value;
        return this
    }

    cgroup(value: string): KafkaConnector {
        this._cgroup = value;
        return this
    }

    clientId(value: string): KafkaConnector {
        this._clientId = value;
        return this
    }

    topic(value: string): KafkaConnector {
        this._topic = value;
        return this
    }

    connect() {
        let config: KafkaConfig = {
            credential: {
                password: this._credential.getPassword(),
                username: this._credential.getUsername()
            },
            topic: this._topic,
            brokers: this._brokers,
            partition: this._partition,
            cgroup: this._cgroup,
            clientId: this._clientId
        }

        fetch(ConnectorCreateUrl, {
            method: POST,
            body: JSON.stringify(
                {
                    'kind': kafkaConnectorType,
                    'data': config
                }
            ),
            headers: ApplicationJsonHeader
        }).then()
    }
}

export class Credential {
    private _username: string
    private _password: string

    public static new(): Credential {
        return new Credential()
    }

    getPassword(): string {
        return this._password;
    }

    getUsername(): string {
        return this._username;
    }

    password(value: string): Credential {
        this._password = value;
        return this
    }

    username(value: string): Credential {
        this._username = value;
        return this
    }
}