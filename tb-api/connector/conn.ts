type FieldMapper = {
    source: string
    target: string
}

export const kafkaConnectorType = "Kafka"

export abstract class Connector {
    mapper: FieldMapper[]
    table: string

    /**
     *
     */
    abstract connect()

    /**
     *
     * @param source_field
     * @param target_field
     */
    map(source_field: string, target_field: string): Connector {
        this.mapper.push({
            source: source_field,
            target: target_field
        })
        return this
    }

    /**
     *
     * @param t
     */
    targetTable(t: string): Connector {
        this.table = t
        return this
    }
}