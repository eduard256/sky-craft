"""
FastAPI auth service for Sky Craft.

Endpoints:
  POST /auth/request-code   - Generate login code, send to Telegram
  POST /auth/verify-code    - Verify code, create persistent session
  POST /auth/validate-token - Check if a session token is still valid
  POST /auth/revoke-token   - Revoke a session by its token
  GET  /auth/sessions/{nickname} - List active sessions
  DELETE /auth/sessions/{session_id} - Delete a specific session
"""

import logging
import os
from datetime import datetime, timezone

from fastapi import FastAPI, HTTPException, Request
from pydantic import BaseModel

import db
from bot import get_bot

logger = logging.getLogger(__name__)

app = FastAPI(title="Sky Craft Auth API", version="0.2.0")


# ── Request/response models ─────────────────────────────────────────────────

class RequestCodeBody(BaseModel):
    nickname: str


class VerifyCodeBody(BaseModel):
    nickname: str
    code: str
    device_info: str = ""


class TokenBody(BaseModel):
    token: str


class DeleteSessionBody(BaseModel):
    nickname: str


class RequestCodeResponse(BaseModel):
    status: str
    message: str


class VerifyCodeResponse(BaseModel):
    status: str
    token: str


class ValidateTokenResponse(BaseModel):
    status: str
    nickname: str


class SessionOut(BaseModel):
    id: int
    ip: str
    device_info: str
    created_at: str


class SessionsListResponse(BaseModel):
    status: str
    sessions: list[SessionOut]
    count: int
    max: int


# ── Endpoints ────────────────────────────────────────────────────────────────

@app.post("/auth/request-code", response_model=RequestCodeResponse)
async def request_code(body: RequestCodeBody) -> RequestCodeResponse:
    """
    Generate an auth code and send it to the player's Telegram.
    Called by the game client when the player enters their nickname.
    """
    nickname = body.nickname.strip()

    telegram_id = await db.get_telegram_id_by_nickname(nickname)
    if telegram_id is None:
        raise HTTPException(
            status_code=404,
            detail="Пользователь не найден. Зарегистрируйся через Telegram бота.",
        )

    code = await db.create_auth_code(nickname)

    bot = get_bot()
    try:
        ttl_minutes = int(os.getenv("AUTH_CODE_TTL_SECONDS", "300")) // 60
        await bot.send_message(
            chat_id=telegram_id,
            text=(
                f"Твой код для входа в Sky Craft: **{code}**\n\n"
                f"Действителен {ttl_minutes} минут.\n"
                "Не сообщай его никому."
            ),
            parse_mode="Markdown",
        )
    except Exception as e:
        logger.error("Failed to send Telegram message to %s: %s", telegram_id, e)
        raise HTTPException(
            status_code=502,
            detail="Не удалось отправить код в Telegram. Попробуй позже.",
        )

    return RequestCodeResponse(status="ok", message="Код отправлен в Telegram.")


@app.post("/auth/verify-code", response_model=VerifyCodeResponse)
async def verify_code(body: VerifyCodeBody, request: Request) -> VerifyCodeResponse:
    """
    Verify the auth code. On success creates a persistent session
    and returns a long-lived token.
    """
    nickname = body.nickname.strip()
    code = body.code.strip()

    valid = await db.verify_auth_code(nickname, code)
    if not valid:
        raise HTTPException(status_code=401, detail="Неверный или просроченный код.")

    # Get client IP from X-Forwarded-For or direct connection
    ip = request.headers.get("X-Forwarded-For", "").split(",")[0].strip()
    if not ip:
        ip = request.client.host if request.client else "unknown"

    # Check session limit
    count = await db.count_sessions(nickname)
    if count >= db.MAX_SESSIONS:
        sessions = await db.get_sessions(nickname)
        session_list = []
        for s in sessions:
            dt = datetime.fromtimestamp(s.created_at, tz=timezone.utc)
            session_list.append(f"  -- {dt.strftime('%d.%m.%Y %H:%M')} | {s.ip} | {s.device_info}")

        raise HTTPException(
            status_code=409,
            detail=(
                f"Максимум {db.MAX_SESSIONS} сессий. "
                "Удали одну из существующих через Telegram бота или клиент.\n"
                + "\n".join(session_list)
            ),
        )

    token = await db.create_session(nickname, ip, body.device_info)

    return VerifyCodeResponse(status="ok", token=token)


@app.post("/auth/validate-token", response_model=ValidateTokenResponse)
async def validate_token(body: TokenBody) -> ValidateTokenResponse:
    """
    Validate a session token. Called by the game server when a player connects.
    Returns 200 with nickname if valid, 401 if revoked/invalid.
    """
    nickname = await db.validate_token(body.token)
    if nickname is None:
        raise HTTPException(status_code=401, detail="Токен недействителен.")

    return ValidateTokenResponse(status="ok", nickname=nickname)


@app.post("/auth/revoke-token")
async def revoke_token(body: TokenBody) -> dict:
    """
    Revoke a session by its token. Called by the game client
    when the player wants to log out.
    """
    deleted = await db.revoke_token(body.token)
    if not deleted:
        raise HTTPException(status_code=404, detail="Сессия не найдена.")

    return {"status": "ok", "message": "Сессия удалена."}


@app.get("/auth/sessions/{nickname}", response_model=SessionsListResponse)
async def list_sessions(nickname: str) -> SessionsListResponse:
    """List all active sessions for a user."""
    telegram_id = await db.get_telegram_id_by_nickname(nickname)
    if telegram_id is None:
        raise HTTPException(status_code=404, detail="Пользователь не найден.")

    sessions = await db.get_sessions(nickname)
    session_list = []
    for s in sessions:
        dt = datetime.fromtimestamp(s.created_at, tz=timezone.utc)
        session_list.append(SessionOut(
            id=s.id,
            ip=s.ip,
            device_info=s.device_info,
            created_at=dt.strftime("%d.%m.%Y %H:%M"),
        ))

    return SessionsListResponse(
        status="ok",
        sessions=session_list,
        count=len(session_list),
        max=db.MAX_SESSIONS,
    )


@app.delete("/auth/sessions/{session_id}")
async def delete_session(session_id: int, body: DeleteSessionBody) -> dict:
    """Delete a specific session by ID. Requires nickname for ownership verification."""
    deleted = await db.delete_session_by_id(session_id, body.nickname)
    if not deleted:
        raise HTTPException(
            status_code=404,
            detail="Сессия не найдена или не принадлежит этому пользователю.",
        )

    return {"status": "ok", "message": "Сессия удалена."}
