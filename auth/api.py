"""
FastAPI auth service for Sky Craft.

Endpoints:
  POST /auth/request-code  { "nickname": "..." }
    -> Generates a 6-digit code, sends it to the user's Telegram, returns 200.
  POST /auth/verify-code   { "nickname": "...", "code": "..." }
    -> Verifies the code, returns 200 with a session token on success.

The game client calls these endpoints when a player connects to a server.
"""

import logging
import os
import secrets

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

import db
from bot import get_bot

logger = logging.getLogger(__name__)

app = FastAPI(title="Sky Craft Auth API", version="0.1.0")


class RequestCodeBody(BaseModel):
    nickname: str


class VerifyCodeBody(BaseModel):
    nickname: str
    code: str


class RequestCodeResponse(BaseModel):
    status: str
    message: str


class VerifyCodeResponse(BaseModel):
    status: str
    session_token: str


@app.post("/auth/request-code", response_model=RequestCodeResponse)
async def request_code(body: RequestCodeBody) -> RequestCodeResponse:
    """
    Generate an auth code and send it to the player's Telegram.
    Called by the game client when the player enters their nickname.
    """
    nickname = body.nickname.strip()

    # Check if user exists
    telegram_id = await db.get_telegram_id_by_nickname(nickname)
    if telegram_id is None:
        raise HTTPException(
            status_code=404,
            detail=f'User "{nickname}" not found. Register via the Telegram bot first.',
        )

    # Generate code and store it
    code = await db.create_auth_code(nickname)

    # Send code to user via Telegram
    bot = get_bot()
    try:
        await bot.send_message(
            chat_id=telegram_id,
            text=(
                f"Your Sky Craft login code: **{code}**\n\n"
                f"Valid for {int(os.getenv('AUTH_CODE_TTL_SECONDS', '300')) // 60} minutes.\n"
                "Do not share this code with anyone."
            ),
            parse_mode="Markdown",
        )
    except Exception as e:
        logger.error("Failed to send Telegram message to %s: %s", telegram_id, e)
        raise HTTPException(
            status_code=502,
            detail="Failed to send code via Telegram. Try again later.",
        )

    return RequestCodeResponse(
        status="ok",
        message="Code sent to your Telegram.",
    )


@app.post("/auth/verify-code", response_model=VerifyCodeResponse)
async def verify_code(body: VerifyCodeBody) -> VerifyCodeResponse:
    """
    Verify the auth code entered by the player.
    Returns a session token on success that the game server uses
    to authenticate the player's connection.
    """
    nickname = body.nickname.strip()
    code = body.code.strip()

    valid = await db.verify_auth_code(nickname, code)
    if not valid:
        raise HTTPException(
            status_code=401,
            detail="Invalid or expired code.",
        )

    # Generate a session token
    session_token = secrets.token_urlsafe(32)

    # TODO: store session_token on server side for validation by game server

    return VerifyCodeResponse(
        status="ok",
        session_token=session_token,
    )
