
//dev: can be implemented using wrapper like in lsp_types crate

/*
Like the language server protocol, the shutdown build request is sent from the client to the server.
It asks the server to shut down, but to not exit (otherwise the response might not be delivered correctly to the client).
There is a separate exit notification that asks the server to exit.

Request:
method: build/shutdown
params: null

Response:
result: null
error: code and message set in case an exception happens during shutdown request.
*/