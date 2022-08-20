import {Sink} from "../sink/sink";

export interface Stream {

    getStreamId(): bigint

    asStreamGraph(): StreamGraph

    /**
     * calculate the sum of Stream incrementally
     */
    sum(): Stream

    /**
     * count the number of Stream incrementally
     */
    count(): Stream

    /**
     * Calculate then average value of Stream incrementally
     */
    avg(): Stream

    /**
     * add()
     * @param stream
     */
    add(stream: Stream): Stream

    /**
     * minus()
     * @param stream
     */
    minus(stream: Stream): Stream

    /**
     * mul()
     * @param stream
     */
    mul(stream: Stream): Stream

    /**
     * div()
     * @param stream
     */
    div(stream: Stream): Stream

    /**
     *
     * @param cond
     */
    sumif(cond: Condition): Stream

    /**
     *
     * @param cond
     */
    countif(cond: Condition): Stream

    /**
     *
     * @param cond
     */
    avgif(cond: Condition): Stream
}

export class ConditionBuilder {
    condition: Condition

    constructor() {
        this.condition = new Condition()
    }

    setLeft(stream: Stream) {
        this.condition.left = stream
    }

    equals(stream: Stream): Condition {
        this.condition.right = stream
        this.condition.op = Operator.Eq
        return this.condition
    }

    unequal(stream: Stream): Condition {
        this.condition.right = stream
        this.condition.op = Operator.Neq
        return this.condition
    }

    less(stream: Stream): Condition {
        this.condition.right = stream
        this.condition.op = Operator.Lt
        return this.condition
    }

    lessThan(stream: Stream): Condition {
        this.condition.right = stream
        this.condition.op = Operator.Lte
        return this.condition
    }

    greater(stream: Stream): Condition {
        this.condition.right = stream
        this.condition.op = Operator.Gt
        return this.condition
    }

    greaterThan(stream: Stream): Condition {
        this.condition.right = stream
        this.condition.op = Operator.Gte
        return this.condition
    }
}

export class Condition {
    left: Stream
    right: Stream
    op: Operator

    public static left(stream: Stream): ConditionBuilder {
        let builder = new ConditionBuilder();
        builder.setLeft(stream)
        return builder
    }

    asStream(): Stream {
        let stream = new OperatorStream(Operator.Eq);
        stream.addUpstream(this.left)
        stream.addUpstream(this.right)
        return stream
    }
}

export class GraphMeta {
    constructor(center: bigint, neighbors: bigint[]) {
        this.center = center;
        this.neighbors = neighbors;
    }

    center: bigint
    neighbors: bigint[]
}


export abstract class SinkableStream implements Stream {
    constructor(idGenerator: NodeIdGenerator) {
        this.streamId = idGenerator.next()
    }

    private _downstream: Stream[]
    private _upstreams: Stream[]
    private readonly streamId: bigint

    addDownstream(stream: Stream): SinkableStream {
        this._downstream.push(stream)
        return this
    }

    addUpstream(stream: Stream): SinkableStream {
        this._upstreams.push(stream)
        return this
    }

    /**
     * sink() is to write this stream incrementally to a specific target sink
     * like fields in Tableflow, MQ, Database or somewhere else.
     * @param sink
     */
    public sink(sink: Sink): void {
        sink.sink(this.asStreamGraph())
    }

    sum(): SinkableStream {
        return this.getOperatorStream(Operator.Sum, null)
    }

    add(stream: Stream): SinkableStream {
        return this.getOperatorStream(Operator.Add, stream)
    }

    div(stream: Stream): SinkableStream {
        return this.getOperatorStream(Operator.Div, stream);
    }

    avg(): SinkableStream {
        return this.getOperatorStream(Operator.Avg, null);
    }

    avgif(cond: Condition): SinkableStream {
        return this.getOperatorStream(Operator.Avgif, cond.asStream());
    }

    count(): SinkableStream {
        return this.getOperatorStream(Operator.Count, null);
    }

    countif(cond: Condition): SinkableStream {
        return this.getOperatorStream(Operator.Countif, cond.asStream());
    }

    minus(stream: Stream): SinkableStream {
        return this.getOperatorStream(Operator.Minus, stream);
    }

    mul(stream: Stream): SinkableStream {
        return this.getOperatorStream(Operator.Mul, stream);
    }

    sumif(cond: Condition): SinkableStream {
        return this.getOperatorStream(Operator.Sumif, cond.asStream());
    }

    asStreamGraph(): StreamGraph {
        let graph = new StreamGraph();
        let meta = this.getMeta();
        graph.meta.push(meta)
        graph.nodes = this.getNodes()
        this._upstreams.forEach(s => {
            let subgraph = s.asStreamGraph();
            graph.merge(subgraph);
        })

        return graph
    }

    private getOperatorStream(operator: Operator, upstream: Stream) {
        let stream = new OperatorStream(operator);
        this.after(stream)
        if (upstream != null) {
            if (upstream instanceof ConstStream) {
                if (this instanceof ConstStream) {
                    let newVar = this as ConstStream<any>;
                    if (typeof newVar.val == "number" && typeof upstream.val == "number") {
                        switch (operator) {
                            case Operator.Add:
                                return ConstStream.new(newVar.val + upstream.val)
                            case Operator.Minus:
                                return ConstStream.new(newVar.val - upstream.val)
                            case Operator.Mul:
                                return ConstStream.new(newVar.val * upstream.val)
                            case Operator.Div:
                                return ConstStream.new(newVar.val / upstream.val)
                        }
                    }
                }
                stream.constStream = upstream
            } else {
                stream.addUpstream(upstream)
            }
        }

        return stream;
    }

    private after<T extends SinkableStream>(stream: SinkableStream) {
        stream.addUpstream(this)
        this.addDownstream(stream)
    }

    private getMeta(): GraphMeta {
        return new GraphMeta(this.streamId, this._downstream.map(s => s.getStreamId()))
    }

    getStreamId(): bigint {
        return this.streamId;
    }

    getNodes(): Map<string, object> {
        return new Map
    }
}

/**
 * In Tableflow, a field of a table is viewed as a stream, it represents Tableflow's core concept 'Dynamic Table as Stream'
 * Field's data change (insert, update, delete) will trigger a new event.
 * The computations refer to fields will be driven by these events and create new events if the results write to another field.
 * So that each Table in Tableflow is Dynamic. You can create a reactive Application easily because
 * computation and storage are integrated naturally and perfectly by Tableflow.
 */
export class FieldStream extends SinkableStream {
    constructor() {
        super(new NodeIdGenerator());
    }

    fieldName: string
    tableName: string

    getNodes(): Map<string, object> {
        let map = new Map();
        map.set(this.getStreamId().toString(), {
            id: this.getStreamId(),
            type: Operator.Reference,
            tableName: this.tableName,
            fieldName: this.fieldName
        })

        return map
    }
}

export class ConstStream<T> extends SinkableStream {
    val: T

    public static new<T>(val: T): ConstStream<T> {
        let stream = new ConstStream<T>(new NodeIdGenerator());
        stream.val = val
        return stream
    }
}

export class OperatorStream extends SinkableStream {
    private _operator: Operator

    constructor(operator: Operator) {
        super(new NodeIdGenerator());
        this._operator = operator;
    }

    private _constStream: ConstStream<any>

    set constStream(value: ConstStream<any>) {
        this._constStream = value;
    }

    getNodes(): Map<string, object> {
        return super.getNodes();
    }
}

enum Operator {
    Add,
    Minus,
    Mul,
    Div,
    Sum,
    Avg,
    Sumif,
    Count,
    Countif,
    Eq,
    Gte,
    Gt,
    Lte,
    Lt,
    Neq,
    Avgif,
    Reference
}

class NodeIdGenerator {
    private currentId: bigint

    next(): bigint {
        return this.currentId++
    }
}

export class StreamGraph {
    meta: GraphMeta[]
    nodes: Map<string, object>

    merge(subgraph: StreamGraph) {
        subgraph.meta.forEach(m => this.meta.push(m))
        subgraph.nodes.forEach((value, key) => this.nodes.set(key, value))
    }
}

