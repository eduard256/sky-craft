"""
Telegram bot for Sky Craft authentication.

Flow:
1. User sends /start
2. Bot asks for a nickname
3. Bot checks uniqueness and registers (nickname <-> telegram_id)
4. When the game client requests a login code via the API,
   the bot sends the code to the user's Telegram
"""

import asyncio
import logging
import os
import re

from aiogram import Bot, Dispatcher, Router, F
from aiogram.types import Message
from aiogram.filters import CommandStart
from aiogram.fsm.context import FSMContext
from aiogram.fsm.state import State, StatesGroup

import db

logger = logging.getLogger(__name__)

router = Router()

# Nickname validation: 3-16 chars, alphanumeric + underscore
NICKNAME_PATTERN = re.compile(r"^[a-zA-Z0-9_]{3,16}$")


class Registration(StatesGroup):
    waiting_for_nickname = State()


@router.message(CommandStart())
async def cmd_start(message: Message, state: FSMContext) -> None:
    """Handle /start command."""
    telegram_id = message.from_user.id

    # Check if already registered
    nickname = await db.get_nickname_by_telegram_id(telegram_id)
    if nickname is not None:
        await message.answer(
            f"You are already registered as **{nickname}**.\n"
            "Login codes will be sent here when you connect to a server.",
            parse_mode="Markdown",
        )
        return

    await state.set_state(Registration.waiting_for_nickname)
    await message.answer(
        "Welcome to **Sky Craft**!\n\n"
        "Choose your nickname (3-16 characters, letters, digits, underscore):",
        parse_mode="Markdown",
    )


@router.message(Registration.waiting_for_nickname)
async def process_nickname(message: Message, state: FSMContext) -> None:
    """Process the nickname the user has chosen."""
    nickname = message.text.strip()

    if not NICKNAME_PATTERN.match(nickname):
        await message.answer(
            "Invalid nickname. Use 3-16 characters: letters, digits, underscore only.\n"
            "Try again:"
        )
        return

    # Check if nickname is taken
    if await db.nickname_exists(nickname):
        await message.answer(
            f'Nickname "{nickname}" is already taken. Choose another one:'
        )
        return

    # Register
    telegram_id = message.from_user.id
    success = await db.register_user(nickname, telegram_id)
    if not success:
        await message.answer(
            "Registration failed (nickname taken or you already have an account). "
            "Try /start again."
        )
        await state.clear()
        return

    await state.clear()
    await message.answer(
        f"Registered as **{nickname}**!\n\n"
        "When you connect to a Sky Craft server, a login code will be sent here.",
        parse_mode="Markdown",
    )


# Global reference to the bot instance so the API can send messages
_bot_instance: Bot | None = None


def get_bot() -> Bot:
    """Get the running bot instance (used by the API to send codes)."""
    if _bot_instance is None:
        raise RuntimeError("Bot is not initialized yet")
    return _bot_instance


async def run_bot() -> None:
    """Initialize and start the bot polling loop."""
    global _bot_instance

    token = os.getenv("TELEGRAM_BOT_TOKEN")
    if not token:
        raise RuntimeError("TELEGRAM_BOT_TOKEN env variable is not set")

    bot = Bot(token=token)
    _bot_instance = bot

    dp = Dispatcher()
    dp.include_router(router)

    logger.info("Starting Telegram bot polling...")
    await dp.start_polling(bot)
