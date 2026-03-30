# TODO: UI & HUD Rendering

## Status: Stub only (HudState struct, no rendering)

## What exists
- `ui.rs`: HudState with show flags, chat history
- `state.rs`: AppState enum (MainMenu, Connecting, Playing)

## What needs to be built

### Text rendering
- Bitmap font from client/assets/textures/minecraft/textures/font/
- Or generate font atlas from TTF at startup
- Render text as textured quads (2D overlay)
- Support: variable width, line wrapping, color codes
- Used by: chat, nametags, debug screen, menus, notifications

### HUD overlay (always visible in-game)
- Rendered as 2D quads on top of 3D world
- No depth testing, fixed screen coordinates
- Uses GUI textures from client/assets/textures/minecraft/textures/gui/

#### Hotbar
- 9 slots, bottom center of screen
- Highlight selected slot
- Show item icon in each slot (from item texture atlas)
- Show item count number (bottom-right of slot)
- Show durability bar (colored bar under item for tools/armor)

#### Health bar
- 10 hearts above hotbar left
- Full/half/empty heart icons
- Flash when taking damage
- Hardcore mode: different heart texture (not applicable, but future)

#### Hunger bar
- 10 shanks above hotbar right
- Full/half/empty icons
- Shake when hunger effect active

#### Armor bar
- Above hearts when wearing armor
- Shield icons representing armor points (0-20)

#### XP bar
- Green progress bar below hotbar
- Level number in center

#### Air bubbles
- Above hunger when underwater
- 10 bubbles, pop one by one as air depletes

#### Crosshair
- Center of screen
- Simple + shape or custom texture

### Sky Craft HUD additions

#### Ring indicator (top-left)
- "Ring N" text with color coding
- Green (0-2), yellow (3-5), orange (6-10), red (11-20), dark red (21+), purple (50+)

#### Island info (below ring)
- Island name + biome when standing on island
- "Void" when in void or on bridge

#### Wind indicator (top-right)
- Arrow showing wind direction
- Strength number in blocks/sec
- Flashes "GUST!" before gust

#### Active debuffs (left side)
- Stack of debuff icons with remaining time
- Unique icon per debuff type

#### Altitude indicator (right side)
- "Y: N" with up/down arrow
- Red flash when near void

### Notification system (center screen)
- Ring transition: "Entering Ring N" with subtitle
- Environmental warnings: "VOID LIGHTNING", "WIND GUST"
- Achievement popups (top-right, slide in/out)
- Death messages (broadcast to all)

### Inventory screen (E key)
- Player inventory grid: 36 slots (4 rows of 9)
- Armor slots (4, left of player model)
- Offhand slot
- 2x2 crafting grid + output
- Player model preview (rotatable)
- Click handling: pick up, place, swap, shift-click, drag

### Container screens
- Chest: 27/54 slots + player inventory
- Furnace: 3 slots + player inventory + progress arrows
- Crafting table: 9+1 grid + player inventory
- Enchanting table: 2 slots + 3 enchantment options
- Anvil: 3 slots + XP cost display
- Brewing stand: 5 slots + brew progress
- Each container type needs unique layout

### Chat
- Text input at bottom of screen (T key to open)
- Message history: scrollable, last 10 sec visible by default
- Tab completion for player names and commands
- Max 256 characters

### Pause menu (ESC)
- Back to Game, Settings, Statistics, Disconnect
- Semi-transparent dark overlay

### Settings screen
- Video: render distance, FPS cap, FOV, clouds, particles, vsync
- Audio: master, music, blocks, mobs, environment, void ambience
- Controls: key rebinding
- Sliders, toggles, dropdown selectors

### Main menu
- Sky Craft title
- Connect to Server button -> server address input
- Settings button
- Quit button
- Recent servers list

### Login screen
- Nickname input
- Request code button -> code input
- Verify button
- Register link to Telegram bot

### Death screen
- "You Died!" title
- Death cause and coordinates
- Score
- Respawn / Title Screen buttons

### Statistics screen
- Highest Ring, Islands Explored, Mobs Killed, Deaths, Distance, Play Time

### Debug screen (F3)
- FPS, position (XYZ), chunk coords, facing direction
- Biome, light level, ring number
- Loaded chunks count, entity count
- Memory usage, GPU info

### Files to create
- `client/src/ui/mod.rs` -- UI render dispatcher
- `client/src/ui/text.rs` -- bitmap font text rendering
- `client/src/ui/hud.rs` -- in-game HUD overlay
- `client/src/ui/inventory.rs` -- inventory/container screens
- `client/src/ui/menu.rs` -- main menu, pause menu, settings
- `client/src/ui/chat.rs` -- chat input and display
- `client/src/ui/notification.rs` -- ring transitions, warnings, achievements
- `client/src/ui/debug.rs` -- F3 debug screen

### Estimated: ~5000 lines
