# WebSocket Auth flow

This repository is a proof of concept for JWT flow without Access Token. \
It is implemented in Rust just to practice this language and does not rely on anything specific from it

## Description

### Fresh client
- client connects through WebSocket
- client sends authentication message
- server checks the authentication message (next steps assume that the message is valid)
- server returns a refresh token
- server flags the client connection as `authenticated` by assigning the refresh token to the session
- client stores the refresh token
- server checks the refresh token assigned to the session in heartbeat calls
- if refresh token is about to expire server sends a new one

### Previously authorized client
- client connects through WebSocket
- client sends the refresh token
- server checks the refresh token (next steps assume that the token is valid)
- server blacklists the refresh token
- server returns the new refresh token
- server assigns the refresh token to the session for future heartbeat checks

## Considerations
### Horizontal scaling
It seems that nothing prevents from having multiple nodes of such service as long as
- the refresh tokens are signed with JWK\secret that's either same or can be independently validated
- blacklist is independently accessible

### Heartbeat
The refresh token check during heartbeat is used only for convenience following a consideration that performance penalty is negligible since the heartbeat is required anyway. \
Another variant is to poractively request token refresh from client side

### Cookies
The refresh token might also be added to the HTTPS cookie, but it would require breaking the WebSocket connection since the server can only set HttpOnly cookie in a response to the initial HTTP GET request.
Implementing this approach can save a single initial message