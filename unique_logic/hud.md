# HUD & UI (All text in English)

## Main HUD Elements (Always Visible)

### Standard MC HUD
- Hotbar: 9 slots, bottom center
- Health bar: 10 hearts, above hotbar left
- Hunger bar: 10 shanks, above hotbar right
- XP bar: green bar below hotbar, level number in center
- Armor bar: above hearts when wearing armor
- Air bubbles: above hunger when underwater

### Sky Craft Additions

#### Ring Indicator (Top-Left Corner)
```
Ring 0 | Safe Zone
Ring 3 | Moderate Danger
Ring 10 | High Danger
Ring 50 | Extreme Danger
```
- Shows current ring number + danger label
- Color: green (0-2), yellow (3-5), orange (6-10), red (11-20), dark red (21+), purple (50+)
- Pulses briefly when crossing ring boundary

#### Island Info (Below Ring Indicator)
```
Misty Forest Haven
Biome: Forest | Size: 340x280
```
- Shows current island name and biome
- Only visible when standing on an island
- When in void/on bridge: shows "Void" instead

#### Wind Indicator (Top-Right Corner)
```
Wind: ← 2.4 b/s
```
- Arrow shows wind direction
- Number shows wind strength in blocks/sec
- Color: white (calm), yellow (moderate), red (dangerous)
- Hidden at ring 0 (no wind)
- Flashes "GUST!" 0.5 sec before gust hits

#### Active Debuffs (Left Side, Below Hearts)
- Stack of icons for mob-applied debuffs
- Each icon has remaining time as small number
- Icons:
  - Placement Lock: crossed-out block icon
  - Mining Lock: crossed-out pickaxe icon
  - Inventory Lock: crossed-out chest icon
  - Gravity Pull: downward arrow icon
  - Fear: skull icon
  - Void Sickness: swirl icon
  - Soul Drain: purple orb icon

#### Altitude Indicator (Right Side)
```
Y: 87 ↑
```
- Current Y position
- Arrow shows if above (↑) or below (↓) current island average height
- Color changes when dangerously close to void (Y < island bottom - 5)
- Flashes red at Y < 0

## Notification System (Center Screen)

### Ring Transition
```
━━━━━━━━━━━━━━━━━━━━━━━━━
  Entering Ring 5
  "Danger increases. Prepare yourself."
━━━━━━━━━━━━━━━━━━━━━━━━━
```
- Large text, fades after 3 sec
- Unique subtitle per ring tier

### Environmental Warnings
```
⚡ VOID LIGHTNING ⚡
```
- Flashes 1 sec before hazard
- Color: yellow for wind, purple for lightning, gray for fog

### Death Messages
```
Player1 fell into the void
Player1 was blown off a bridge by wind
Player1 was struck by void lightning
Player1 was killed by Zombie [Ring 15]
```
- Custom death messages for Sky Craft-specific deaths
- Ring number shown in mob kill messages
- Broadcast to all players on server

### Achievement-Style Notifications
```
Ring 5 Reached!
"The wind grows stronger here..."

First Diamond Found!
"Deep within the island core..."

100 Islands Explored!
"Cartographer of the skies"
```
- Pop up top-right, slide in and out
- Custom milestones for Sky Craft progression

## Screens

### Main Menu (Client)
```
┌─────────────────────────────┐
│                             │
│        SKY CRAFT            │
│                             │
│   ┌───────────────────┐     │
│   │ Connect to Server │     │
│   └───────────────────┘     │
│   ┌───────────────────┐     │
│   │     Settings      │     │
│   └───────────────────┘     │
│   ┌───────────────────┐     │
│   │       Quit        │     │
│   └───────────────────┘     │
│                             │
│  v0.1.0                     │
└─────────────────────────────┘
```

### Server Connect Screen
```
┌─────────────────────────────┐
│  Connect to Server          │
│                             │
│  Address: [192.168.1.5    ] │
│  Port:    [35565          ] │
│                             │
│  Recent Servers:            │
│  > My Server (192.168.1.5)  │
│  > Friend's (10.0.0.42)    │
│                             │
│  [Connect]       [Back]     │
└─────────────────────────────┘
```

### Login Screen (No Token)
```
┌─────────────────────────────┐
│  Login                      │
│                             │
│  Nickname: [player1       ] │
│                             │
│  [Request Code]             │
│                             │
│  Enter code from Telegram:  │
│  Code: [______]             │
│                             │
│  [Verify]        [Back]     │
│                             │
│  Register at @skycraftauth_bot │
└─────────────────────────────┘
```

### Pause Menu (In-Game, ESC)
```
┌─────────────────────────────┐
│  Game Menu                  │
│                             │
│  [Back to Game]             │
│  [Settings]                 │
│  [Statistics]               │
│  [Disconnect]               │
└─────────────────────────────┘
```

### Statistics Screen
```
┌─────────────────────────────┐
│  Player Statistics          │
│                             │
│  Highest Ring Reached:  12  │
│  Islands Explored:      47  │
│  Bridges Built (blocks): 2340 │
│  Mobs Killed:          184  │
│  Deaths:                23  │
│  Void Deaths:           15  │
│  Wind Deaths:            4  │
│  Time Played:        14h30m │
│  Distance Walked:    12.4km │
│                             │
│  [Close]                    │
└─────────────────────────────┘
```

### Settings Screen
```
Categories:
  Video: render distance, FPS cap, fancy graphics, clouds, particles
  Audio: master, music, blocks, mobs, environment, void ambience, weather
  Controls: key bindings (standard MC layout)
  Game: chat visibility, HUD scale, language (English only V1)
```

### Death Screen
```
┌─────────────────────────────┐
│                             │
│        You Died!            │
│                             │
│  Blown off bridge by wind   │
│  Ring 8 | Y: -12            │
│                             │
│  Score: 247                 │
│                             │
│  [Respawn]                  │
│  [Title Screen]             │
└─────────────────────────────┘
```

## Chat System
- Open: T key
- Command prefix: /
- Chat window: bottom-left, 10 visible lines
- Messages fade after 10 sec, scrollable with mouse wheel
- Tab completion for player names and commands
- Max message length: 256 characters
