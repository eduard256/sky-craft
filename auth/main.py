"""
Entry point for Sky Craft auth service.
Runs the Telegram bot and the FastAPI server concurrently.
"""

import asyncio
import logging
import os

import uvicorn
from dotenv import load_dotenv

load_dotenv()

import db
from bot import run_bot
from api import app

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
)
logger = logging.getLogger(__name__)


async def run_api() -> None:
    """Run the FastAPI server."""
    host = os.getenv("AUTH_API_HOST", "0.0.0.0")
    port = int(os.getenv("AUTH_API_PORT", "8080"))

    config = uvicorn.Config(app, host=host, port=port, log_level="info")
    server = uvicorn.Server(config)
    await server.serve()


async def main() -> None:
    """Initialize DB and run both the bot and the API concurrently."""
    logger.info("Initializing database...")
    await db.init_db()

    logger.info("Starting Sky Craft auth service...")
    await asyncio.gather(
        run_bot(),
        run_api(),
    )


if __name__ == "__main__":
    asyncio.run(main())
