import winston from "winston";

export const POST = "POST";

export var ENDPOINT = process.env.LIGHTFLUS_ENDPOINT;

export const createResourceApiEndpoint = `http://${ENDPOINT}/resources/create`;
export const ApplicationJson = "application/json";
export const ApplicationStream = "application/octet-stream";
export const ContentType = "Content-Type";

export const logger = winston.createLogger({
  level: "info",
  format: winston.format.json(),
  transports: [
    //
    // - Write all logs with importance level of `error` or less to `error.log`
    // - Write all logs with importance level of `info` or less to `combined.log`
    //
    new winston.transports.Console()
  ]
});