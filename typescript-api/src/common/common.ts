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
  let indexOf = func.indexOf(prefix);
  return prefix.concat(name.concat(" ").concat(func.slice(indexOf + prefix.length)));
}