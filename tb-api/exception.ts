export class TableflowException implements Error {
    code: string
    message: string
    name: string;

    constructor(code: string, message: string) {
        this.code = code;
        this.message = message;
        this.name = "tableflow-exception"
    }
}

export const DuplicatedKeyCode = "DuplicateKey"
export const StreamDeployFailed = "StreamDeployFailed"