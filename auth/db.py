"""
Database layer for Sky Craft authentication.
Stores user accounts (nickname <-> telegram_id) and temporary auth codes.
Uses SQLite via aiosqlite for async access.
"""

import aiosqlite
import os
import time
import secrets
import string
from pathlib import Path


DATABASE_PATH = os.getenv("DATABASE_PATH", "./data/auth.db")
AUTH_CODE_TTL = int(os.getenv("AUTH_CODE_TTL_SECONDS", "300"))


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
        await db.commit()


async def nickname_exists(nickname: str) -> bool:
    """Check if a nickname is already taken."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "SELECT 1 FROM users WHERE nickname = ?", (nickname,)
        )
        row = await cursor.fetchone()
        return row is not None


async def telegram_id_registered(telegram_id: int) -> bool:
    """Check if a telegram user already has an account."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "SELECT 1 FROM users WHERE telegram_id = ?", (telegram_id,)
        )
        row = await cursor.fetchone()
        return row is not None


async def get_nickname_by_telegram_id(telegram_id: int) -> str | None:
    """Get the nickname associated with a telegram_id."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "SELECT nickname FROM users WHERE telegram_id = ?", (telegram_id,)
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


async def get_telegram_id_by_nickname(nickname: str) -> int | None:
    """Get the telegram_id for a given nickname."""
    async with aiosqlite.connect(DATABASE_PATH) as db:
        cursor = await db.execute(
            "SELECT telegram_id FROM users WHERE nickname = ?", (nickname,)
        )
        row = await cursor.fetchone()
        return row[0] if row else None


def generate_code(length: int = 6) -> str:
    """Generate a random numeric auth code."""
    return "".join(secrets.choice(string.digits) for _ in range(length))


async def create_auth_code(nickname: str) -> str:
    """
    Create a new auth code for the given nickname.
    Invalidates any previous unused codes for this nickname.
    Returns the new code.
    """
    code = generate_code()
    async with aiosqlite.connect(DATABASE_PATH) as db:
        # Invalidate old codes
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
            # Code expired, mark as used
            await db.execute("UPDATE auth_codes SET used = 1 WHERE id = ?", (code_id,))
            await db.commit()
            return False

        # Valid code, mark as used
        await db.execute("UPDATE auth_codes SET used = 1 WHERE id = ?", (code_id,))
        await db.commit()
        return True
