"""
Telegram bot for Sky Craft authentication (Russian language).

Commands:
  /start - Registration flow (choose nickname)

Keyboard:
  "Мои сессии" button - List active sessions with inline buttons to revoke them

When the game client requests a login code via the API,
the bot sends the code to the user's Telegram.
"""

import logging
import os
import re
from datetime import datetime, timezone

from aiogram import Bot, Dispatcher, Router, F
from aiogram.types import (
    Message,
    CallbackQuery,
    ReplyKeyboardMarkup,
    KeyboardButton,
    InlineKeyboardMarkup,
    InlineKeyboardButton,
)
from aiogram.filters import CommandStart
from aiogram.fsm.context import FSMContext
from aiogram.fsm.state import State, StatesGroup

import db

logger = logging.getLogger(__name__)

router = Router()

# Nickname validation: 3-16 chars, alphanumeric + underscore
NICKNAME_PATTERN = re.compile(r"^[a-zA-Z0-9_]{3,16}$")

# Persistent reply keyboard shown after registration
MAIN_KEYBOARD = ReplyKeyboardMarkup(
    keyboard=[[KeyboardButton(text="Мои сессии")]],
    resize_keyboard=True,
)


class Registration(StatesGroup):
    waiting_for_nickname = State()


@router.message(CommandStart())
async def cmd_start(message: Message, state: FSMContext) -> None:
    """Handle /start command."""
    telegram_id = message.from_user.id

    nickname = await db.get_nickname_by_telegram_id(telegram_id)
    if nickname is not None:
        await message.answer(
            f"Ты уже зарегистрирован как **{nickname}**.\n"
            "Коды для входа будут приходить сюда.",
            parse_mode="Markdown",
            reply_markup=MAIN_KEYBOARD,
        )
        return

    await state.set_state(Registration.waiting_for_nickname)
    await message.answer(
        "Добро пожаловать в **Sky Craft**!\n\n"
        "Придумай ник (3-16 символов, латинские буквы, цифры, подчёркивание):",
        parse_mode="Markdown",
    )


@router.message(Registration.waiting_for_nickname)
async def process_nickname(message: Message, state: FSMContext) -> None:
    """Process the nickname the user has chosen."""
    nickname = message.text.strip()

    if not NICKNAME_PATTERN.match(nickname):
        await message.answer(
            "Неверный формат. Используй 3-16 символов: латинские буквы, цифры, подчёркивание.\n"
            "Попробуй ещё:"
        )
        return

    if await db.nickname_exists(nickname):
        await message.answer(
            f'Ник "{nickname}" уже занят. Выбери другой:'
        )
        return

    telegram_id = message.from_user.id
    success = await db.register_user(nickname, telegram_id)
    if not success:
        await message.answer(
            "Ошибка регистрации (ник занят или у тебя уже есть аккаунт). "
            "Попробуй /start заново."
        )
        await state.clear()
        return

    await state.clear()
    await message.answer(
        f"Зарегистрирован как **{nickname}**!\n\n"
        "Когда подключишься к серверу Sky Craft, "
        "код для входа придёт сюда.",
        parse_mode="Markdown",
        reply_markup=MAIN_KEYBOARD,
    )


@router.message(F.text == "Мои сессии")
async def show_sessions(message: Message) -> None:
    """Show active sessions with inline buttons to revoke them."""
    telegram_id = message.from_user.id
    nickname = await db.get_nickname_by_telegram_id(telegram_id)

    if nickname is None:
        await message.answer(
            "Ты не зарегистрирован. Напиши /start для регистрации."
        )
        return

    sessions = await db.get_sessions(nickname)

    if not sessions:
        await message.answer("У тебя нет активных сессий.")
        return

    buttons = []
    text_lines = [f"Активные сессии ({len(sessions)}/{db.MAX_SESSIONS}):\n"]

    for s in sessions:
        dt = datetime.fromtimestamp(s.created_at, tz=timezone.utc)
        date_str = dt.strftime("%d.%m.%Y %H:%M")
        label = f"{date_str} | {s.ip}"
        if s.device_info:
            label += f" | {s.device_info}"

        text_lines.append(f"-- {label}")
        buttons.append([
            InlineKeyboardButton(
                text=f"Удалить: {date_str} | {s.ip}",
                callback_data=f"revoke:{s.id}",
            )
        ])

    keyboard = InlineKeyboardMarkup(inline_keyboard=buttons)
    await message.answer(
        "\n".join(text_lines),
        reply_markup=keyboard,
    )


@router.callback_query(F.data.startswith("revoke:"))
async def revoke_session_callback(callback: CallbackQuery) -> None:
    """Handle session revoke button press."""
    telegram_id = callback.from_user.id
    nickname = await db.get_nickname_by_telegram_id(telegram_id)

    if nickname is None:
        await callback.answer("Ошибка: аккаунт не найден.", show_alert=True)
        return

    session_id_str = callback.data.split(":", 1)[1]
    try:
        session_id = int(session_id_str)
    except ValueError:
        await callback.answer("Ошибка: неверный ID сессии.", show_alert=True)
        return

    deleted = await db.delete_session_by_id(session_id, nickname)
    if deleted:
        await callback.answer("Сессия удалена!")
    else:
        await callback.answer("Сессия не найдена (уже удалена?).", show_alert=True)

    # Refresh the session list in the same message
    sessions = await db.get_sessions(nickname)

    if not sessions:
        await callback.message.edit_text("У тебя нет активных сессий.")
        return

    buttons = []
    text_lines = [f"Активные сессии ({len(sessions)}/{db.MAX_SESSIONS}):\n"]

    for s in sessions:
        dt = datetime.fromtimestamp(s.created_at, tz=timezone.utc)
        date_str = dt.strftime("%d.%m.%Y %H:%M")
        label = f"{date_str} | {s.ip}"
        if s.device_info:
            label += f" | {s.device_info}"

        text_lines.append(f"-- {label}")
        buttons.append([
            InlineKeyboardButton(
                text=f"Удалить: {date_str} | {s.ip}",
                callback_data=f"revoke:{s.id}",
            )
        ])

    keyboard = InlineKeyboardMarkup(inline_keyboard=buttons)
    await callback.message.edit_text(
        "\n".join(text_lines),
        reply_markup=keyboard,
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
