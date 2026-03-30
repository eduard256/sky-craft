# Sky Craft Auth Service

Centralized authentication service for Sky Craft. Handles player registration via Telegram bot
and session management via REST API. All game servers use a single shared auth instance --
server admins do not need to configure anything auth-related.

## Architecture Overview

```
+----------------+       +-------------------+       +------------------+
|  Game Client   | <---> |   Game Server     | <---> |   Auth API       |
|  (Rust/wgpu)   |       |   (Docker)        |       | (FastAPI + Bot)  |
+----------------+       +-------------------+       +------------------+
       |                                                      |
       |  POST /auth/request-code                             |
       |  POST /auth/verify-code                              |
       |----------------------------------------------------->|
       |                                                      |
       |                  POST /auth/validate-token           |
       |                  |------------------------------>    |
       |                                                      |
       +------------------------------------------------------+
                              |
                      +-------v--------+
                      | Telegram Bot   |
                      | @skycraftauth  |
                      +----------------+
```

**Key principle:** Auth API is a single centralized service at `apiskycraft.webaweba.com`.
Game servers only call `POST /auth/validate-token` to verify players. Server admins just
run the game server Docker container and forward ports -- zero auth configuration needed.

## Player Registration Flow

1. Player opens Telegram bot `@skycraftauth_bot` and sends `/start`
2. Bot asks for a nickname (3-16 chars, `[a-zA-Z0-9_]`)
3. Bot checks uniqueness and registers the player (stores `nickname <-> telegram_id`)
4. Bot shows a keyboard with "Мои сессии" button

Registration happens once. After that the player can log in from up to 5 devices.

## Login Flow (Game Client)

### First login on a new device

```
Client                          Auth API                    Telegram Bot
  |                                |                            |
  |  POST /auth/request-code      |                            |
  |  { "nickname": "player1" }    |                            |
  |------------------------------->|                            |
  |                                |  Send 6-digit code         |
  |                                |--------------------------->|
  |  200 { "message": "..." }     |                            |
  |<-------------------------------|                            |
  |                                |                            |
  |  Player reads code in TG      |                            |
  |                                |                            |
  |  POST /auth/verify-code       |                            |
  |  { "nickname": "player1",     |                            |
  |    "code": "154303",          |                            |
  |    "device_info": "Win 11" }  |                            |
  |------------------------------->|                            |
  |                                |                            |
  |  200 { "token": "abc..." }    |                            |
  |<-------------------------------|                            |
  |                                |                            |
  |  Save token to disk           |                            |
  |  ~/.skycraft/session.json     |                            |
```

After successful verification, the client receives a **permanent session token** (no expiration).
The client stores it locally and reuses it for all future connections.

### Subsequent logins (token exists on disk)

```
Client                     Game Server                  Auth API
  |                            |                            |
  |  Connect + send token      |                            |
  |--------------------------->|                            |
  |                            |  POST /auth/validate-token |
  |                            |  { "token": "abc..." }     |
  |                            |--------------------------->|
  |                            |                            |
  |                            |  200 { "nickname": "..." } |
  |                            |<---------------------------|
  |                            |                            |
  |  Connection accepted       |                            |
  |<---------------------------|                            |
```

If validate-token returns `401`, the client deletes the local token file and shows
the login screen again.

## Session Management

Each player can have up to **5 active sessions** (configurable via `MAX_SESSIONS` env var).
When the limit is reached, the player must revoke an existing session before logging in
from a new device.

Sessions can be revoked from:
- **Telegram bot** -- press "Мои сессии" button, then tap the session to delete
- **Game client** -- call `POST /auth/revoke-token` with the local token
- **Game client** -- call `DELETE /auth/sessions/{session_id}` to remove a specific session

When a session is revoked, the token is deleted from the database. The next time the game
server calls `validate-token` with that token, it gets `401`. The game server then
disconnects the player, and the client shows the login screen.

## API Reference

Base URL: `https://apiskycraft.webaweba.com`

### POST /auth/request-code

Generate a 6-digit login code and send it to the player's Telegram.

**Request:**
```json
{
  "nickname": "player1"
}
```

**Response (200):**
```json
{
  "status": "ok",
  "message": "Код отправлен в Telegram."
}
```

**Errors:**
- `404` -- player not registered (needs to use Telegram bot first)
- `502` -- failed to send Telegram message

---

### POST /auth/verify-code

Verify the login code and create a persistent session.

**Request:**
```json
{
  "nickname": "player1",
  "code": "154303",
  "device_info": "Windows 11 Desktop"
}
```

`device_info` is optional but recommended -- it helps players identify their sessions.
The client IP is captured automatically from the request.

**Response (200):**
```json
{
  "status": "ok",
  "token": "1LNaPkdQcTglLZiXSoGC0Lejcc9g7EAGk9tKcjrd0Zu_Q4lhEp5enz0KyCSmrKmu"
}
```

**Errors:**
- `401` -- invalid or expired code (codes expire after 5 minutes)
- `409` -- session limit reached, response includes list of existing sessions

---

### POST /auth/validate-token

Check if a session token is still valid. **This is the primary endpoint for game servers.**
Called once when a player connects.

**Request:**
```json
{
  "token": "1LNaPkdQcTglLZiXSoGC0Lejcc9g7EAGk9tKcjrd0Zu_Q4lhEp5enz0KyCSmrKmu"
}
```

**Response (200):**
```json
{
  "status": "ok",
  "nickname": "player1"
}
```

**Errors:**
- `401` -- token is invalid or has been revoked

---

### POST /auth/revoke-token

Revoke a session by its token. Used by the game client to log out.

**Request:**
```json
{
  "token": "1LNaPkdQcTglLZiXSoGC0Lejcc9g7EAGk9tKcjrd0Zu_Q4lhEp5enz0KyCSmrKmu"
}
```

**Response (200):**
```json
{
  "status": "ok",
  "message": "Сессия удалена."
}
```

**Errors:**
- `404` -- session not found (already revoked)

---

### GET /auth/sessions/{nickname}

List all active sessions for a player. Used by both the Telegram bot and the game client.

**Response (200):**
```json
{
  "status": "ok",
  "sessions": [
    {
      "id": 1,
      "ip": "192.168.1.5",
      "device_info": "Windows 11 Desktop",
      "created_at": "30.03.2026 11:08"
    },
    {
      "id": 2,
      "ip": "10.0.0.42",
      "device_info": "Linux Laptop",
      "created_at": "29.03.2026 18:30"
    }
  ],
  "count": 2,
  "max": 5
}
```

**Errors:**
- `404` -- player not found

---

### DELETE /auth/sessions/{session_id}

Delete a specific session by its database ID. Requires nickname for ownership verification.

**Request:**
```json
{
  "nickname": "player1"
}
```

**Response (200):**
```json
{
  "status": "ok",
  "message": "Сессия удалена."
}
```

**Errors:**
- `404` -- session not found or does not belong to this player

## Integration Guide

### For Game Client Developers (Rust)

The client must implement the following auth flow:

```
1. On startup:
   - Check if ~/.skycraft/session.json exists
   - If yes: read token, proceed to server connection
   - If no: show login screen

2. Login screen:
   - Player enters nickname
   - Client calls POST /auth/request-code
   - Player enters 6-digit code from Telegram
   - Client calls POST /auth/verify-code with nickname + code + device_info
   - On success: save token to ~/.skycraft/session.json
   - On 409 (session limit): show session list, let player delete one, retry

3. Connecting to game server:
   - Client sends token in the initial handshake packet
   - Game server validates token via auth API
   - If server responds with auth failure: delete local session.json, show login screen

4. Logout:
   - Client calls POST /auth/revoke-token
   - Delete local session.json
   - Show login screen
```

**session.json format:**
```json
{
  "nickname": "player1",
  "token": "1LNaPkdQcTglLZiXSoGC0Lejcc9g7EAGk9tKcjrd0Zu_Q4lhEp5enz0KyCSmrKmu"
}
```

### For Game Server Developers (Rust)

The game server only needs to call one endpoint: `POST /auth/validate-token`.

```
1. Player connects and sends token in handshake
2. Server calls POST /auth/validate-token { "token": "..." }
3. If 200: allow connection, use returned nickname as player identity
4. If 401: reject connection with "Invalid session" message
```

The auth API URL should be hardcoded to `https://apiskycraft.webaweba.com` --
all Sky Craft servers use the same centralized auth. No configuration needed.

### For Server Admins

**You do not need to set up auth.** The auth service runs centrally. Just:

1. Pull and run the game server Docker image:
   ```bash
   docker run -d -p 25565:25565 skycraft/server:latest
   ```
2. Forward port `25565` (or your chosen port) if players connect from the internet
3. Share your `ip:port` with players

That's it. The game server talks to the central auth API automatically.
Players register and manage sessions through the Telegram bot `@skycraftauth_bot`.

## Development Setup

To run the auth service locally for development:

```bash
cd auth/

# Create .env from template
cp .env.example .env
# Edit .env -- set your TELEGRAM_BOT_TOKEN

# Run with Docker
docker build -t skycraft-auth .
docker run -d --name skycraft-auth \
  --env-file .env \
  -p 8080:8080 \
  -v ./data:/app/data \
  skycraft-auth

# Or run directly
pip install -r requirements.txt
python main.py
```

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `TELEGRAM_BOT_TOKEN` | required | Bot token from @BotFather |
| `AUTH_API_HOST` | `0.0.0.0` | API listen address |
| `AUTH_API_PORT` | `8080` | API listen port |
| `DATABASE_PATH` | `./data/auth.db` | SQLite database path |
| `AUTH_CODE_TTL_SECONDS` | `300` | Login code expiration (seconds) |
| `MAX_SESSIONS` | `5` | Max concurrent sessions per player |
