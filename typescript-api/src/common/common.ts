export type AdjacentList = Array<Node>

export class Node {
  centerId: number;
  neighbors: Array<number>;

  constructor(centerId: number, neighbors: number[]) {
    this.centerId = centerId;
    this.neighbors = new Array<number>(...neighbors);
  }
}


export function injectFunctionName(name: string, func: string): string {
  let prefix = "function ";
  let split_start = func.indexOf("=>");
  let split_end = split_start + 2;
  let afterSpit = func.slice(split_end);
  if (afterSpit.trim().indexOf("{") != 0) {
    afterSpit = " { ".concat("return ").concat(afterSpit).concat(" }");
  }
  let beforeSplit = func.slice(0, split_start);
  if (beforeSplit.trim().indexOf("(") != 0) {
    beforeSplit = "(".concat(beforeSplit.trim()).concat(")");
  }

  return prefix.concat(name.concat(" ")).concat(beforeSplit).concat(afterSpit);
}