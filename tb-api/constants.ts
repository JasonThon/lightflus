export const POST = 'POST'
export const GET = 'GET'
export const DELETE = 'DELETE'
export const PATCH = 'PATCH'
export const PUT = 'PUT'
export const ContentType = 'Content-Type'
export const ApplicationJson = 'application/json'
export var Address = 'localhost:8080'
export var TableCreateUrl = `http://${Address}/table/${workspaceId}/create`
export var StreamGraphCreate = `http://${Address}/stream/graph/${workspaceId}/create`
export var ConnectorCreateUrl = `http://${Address}/connector/${workspaceId}/create`
export var workspaceId = 'workspaceId'
export var ApplicationJsonHeader = {
    'Content-Type': ApplicationJson
}

export const StatusCreated = 201