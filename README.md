## Ping Pong Game by using Tcp Sockets in Rust

A TCP server-client application as a ping pong game. You can configure number of clients via 
CLIENT_COUNT. All games will run concurrently.

The client can send one of "Ping", "Pong" or "Miss" in a loop to the server.
Client sends these messages after reading a line from the server. Client also
sends of these messages at the start without waiting for the server's response so that the
communication can start.

MISS_CHANCE is the chance of sending a "Miss" message between 0.0 and 1.0.

With MISS_CHANCE*100 percent change, a Miss is sent by the client.
With (1-MISS_CHANCE)*100/2 percent change, a Ping is sent by the client.
With (1-MISS_CHANCE)*100/2 percent change, a Pong is sent by the client.

Server sends Ping to a Pong message, and a Pong to a Ping message. 
If a Miss is sent by the client, server increments the score of
the entity who sent the last Ping.

Server waits SLEEP_MILLIS_AFTER_SCORING milliseconds after the client or the server scores.

When score of one entity reaches SCORE_TO_WIN, the game ends,
and the server sends a "GameOver" message, which will make client stop.
