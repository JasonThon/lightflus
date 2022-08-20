import {StreamGraph} from "../stream/stream";

export interface Sink {
    sink(graph: StreamGraph): void;
}