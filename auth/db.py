"""
Database layer for Sky Craft authentication.
Stores user accounts (nickname <-> telegram_id) and persistent sessions.
Uses SQLite via aiosqlite for async access.
"""

import aiosqlite
import os
import time
import secrets
import string
from pathlib import Path
from dataclasses import dataclass


DATABASE_PATH = os.getenv("DATABASE_PATH", "./data/auth.db")
AUTH_CODE_TTL = int(os.getenv("AUTH_CODE_TTL_SECONDS", "300"))
MAX_SESSIONS = int(os.getenv("MAX_SESSIONS", "5"))


@dataclass
class Session:
    id: int
    nickname: str
    token: str
    ip: str
    device_info: str
    created_at: float


async def init_db() -> None:
    """Create tables if they don't exist."""
    db_path = Path(DATABASE_PATH)
    db_path.parent.mkdir(parents=True, exist_ok=True)

    async with aiosqlite.connect(DATABASE_PATH) as db:
        await db.execute("""
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                nickname TEXT UNIQUE NOT NULL COLLATE NOCASE,
                telegram_id INTEGER UNIQUE NOT NULL,
                created_at REAL NOT NULL
            )
        """)
        await db.execute("""
            CREATE TABLE IF NOT EXISTS auth_codes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                nickname TEXT NOT NULL COLLATE NOCASE,
                code TEXT NOT NULL,
                created_at REAL NOT NULL,
                used INTEGER NOT NULL DEFAULT 0
            )
        """)
        await db.execute("""
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                nickname TEXT NOT NULL COLLATE NOCASE,
                token TEXT UNIQUE NOT NULL,
                ip TEXT NOT NULL DEFAULT '',
                device_info TEXT NOT NULL DEFAULT '',
                created_at REAL NOT NULL
            )
        """)
        await db.execute("""
            CREATE INDEX IF NOT EXISTS idx_sessions_nickname ON sessions(nickname)
        """)
        await db.execute("""
            CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token)
        """)
        await db.commit()


# ── User operations ──────────────────────────────────────────────────────────

async def nickname_exists(nickname: str) -> bool:
    """Check if a nickname is already taken."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "SELECT 1 FROM users WHERE nickname = ?", (nickname,)
        )
        return await cursor.fetchone() is not None


async def telegram_id_registered(telegram_id: int) -> bool:
    """Check if a telegram user already has an account."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "SELECT 1 FROM users WHERE telegram_id = ?", (telegram_id,)
        )
        return await cursor.fetchone() is not None


async def get_nickname_by_telegram_id(telegram_id: int) -> str | None:
    """Get the nickname associated with a telegram_id."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "SELECT nickname FROM users WHERE telegram_id = ?", (telegram_id,)
        )
        row = await cursor.fetchone()
        return row[0] if row else None


async def get_telegram_id_by_nickname(nickname: str) -> int | None:
    """Get the telegram_id for a given nickname."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "SELECT telegram_id FROM users WHERE nickname = ?", (nickname,)
        )
        row = await cursor.fetchone()
        return row[0] if row else None


async def register_user(nickname: str, telegram_id: int) -> bool:
    """
    Register a new user. Returns True on success, False if nickname
    or telegram_id already exists.
    """
    try:
        async with aiosqlite.connect(DATABASE_PATH) as db:
            await db.execute(
                "INSERT INTO users (nickname, telegram_id, created_at) VALUES (?, ?, ?)",
                (nickname, telegram_id, time.time()),
            )
            await db.commit()
            return True
    except aiosqlite.IntegrityError:
        return False


# ── Auth code operations ─────────────────────────────────────────────────────

def _generate_code(length: int = 6) -> str:
    """Generate a random numeric auth code."""
    return "".join(secrets.choice(string.digits) for _ in range(length))


async def create_auth_code(nickname: str) -> str:
    """
    Create a new auth code for the given nickname.
    Invalidates any previous unused codes for this nickname.
    Returns the new code.
    """
    code = _generate_code()
    async with aiosqlite.connect(DATABASE_PATH) as db:
        await db.execute(
            "UPDATE auth_codes SET used = 1 WHERE nickname = ? AND used = 0",
            (nickname,),
        )
        await db.execute(
            "INSERT INTO auth_codes (nickname, code, created_at) VALUES (?, ?, ?)",
            (nickname, code, time.time()),
        )
        await db.commit()
    return code


async def verify_auth_code(nickname: str, code: str) -> bool:
    """
    Verify an auth code. Returns True if the code is valid and not expired.
    Marks the code as used on success.
    """
    now = time.time()
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            """
            SELECT id, created_at FROM auth_codes
            WHERE nickname = ? AND code = ? AND used = 0
            ORDER BY created_at DESC LIMIT 1
            """,
            (nickname, code),
        )
        row = await cursor.fetchone()
        if row is None:
            return False

        code_id, created_at = row
        if now - created_at > AUTH_CODE_TTL:
            await db.execute("UPDATE auth_codes SET used = 1 WHERE id = ?", (code_id,))
            await db.commit()
            return False

        await db.execute("UPDATE auth_codes SET used = 1 WHERE id = ?", (code_id,))
        await db.commit()
        return True


# ── Session operations ───────────────────────────────────────────────────────

async def count_sessions(nickname: str) -> int:
    """Count active sessions for a user."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "SELECT COUNT(*) FROM sessions WHERE nickname = ?", (nickname,)
        )
        row = await cursor.fetchone()
        return row[0]


async def create_session(nickname: str, ip: str, device_info: str) -> str:
    """
    Create a new persistent session. Returns the session token.
    Raises ValueError if the user already has MAX_SESSIONS active sessions.
    """
    count = await count_sessions(nickname)
    if count >= MAX_SESSIONS:
        raise ValueError(f"Maximum {MAX_SESSIONS} sessions reached")

    token = secrets.token_urlsafe(48)
    async with aiosqlite.connect(DATABASE_PATH) as db:
        await db.execute(
            """
            INSERT INTO sessions (nickname, token, ip, device_info, created_at)
            VALUES (?, ?, ?, ?, ?)
            """,
            (nickname, token, ip, device_info, time.time()),
        )
        await db.commit()
    return token


async def validate_token(token: str) -> str | None:
    """
    Validate a session token. Returns the nickname if valid, None if revoked/invalid.
    """
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "SELECT nickname FROM sessions WHERE token = ?", (token,)
        )
        row = await cursor.fetchone()
        return row[0] if row else None


async def revoke_token(token: str) -> bool:
    """Revoke a session by token. Returns True if a session was deleted."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute("DELETE FROM sessions WHERE token = ?", (token,))
        await db.commit()
        return cursor.rowcount > 0


async def get_sessions(nickname: str) -> list[Session]:
    """Get all active sessions for a user, ordered by creation date."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            """
            SELECT id, nickname, token, ip, device_info, created_at
            FROM sessions WHERE nickname = ?
            ORDER BY created_at DESC
            """,
            (nickname,),
        )
        rows = await cursor.fetchall()
        return [
            Session(id=r[0], nickname=r[1], token=r[2], ip=r[3],
                    device_info=r[4], created_at=r[5])
            for r in rows
        ]


async def delete_session_by_id(session_id: int, nickname: str) -> bool:
    """
    Delete a session by its ID. Requires nickname for ownership check.
    Returns True if a session was deleted.
    """
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "DELETE FROM sessions WHERE id = ? AND nickname = ?",
            (session_id, nickname),
        )
        await db.commit()
        return cursor.rowcount > 0
