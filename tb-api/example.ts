import {DataType, Table} from "./table/table";
import {ConstStream} from "./stream/stream";

function create_table_example() {
    let table = Table.new("table")
        .addField("f1", DataType.INT)
        .addField("f2", DataType.DOUBLE);
    // create table
    table.create()
}

function sum_example() {
    let table = Table.new("table");

    let stream = table.field("f1")
        .stream()
        .sum();

    stream.sink(Table.new("table1").field("f2"))
}

function add_example() {
    let table = Table.new("table");
    // sum(f1) + 1
    table.field("f1")
        .stream()
        .sum()
        .add(ConstStream.new(1))

    table.field("f2")
        .stream()
        .sum()
        .add(Table.new("table").field("f3").stream())
}

add_example()

console.log("add example")