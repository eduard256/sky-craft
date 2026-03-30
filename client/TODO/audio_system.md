# TODO: Audio System

## Status: Stub (AudioManager struct with empty methods)

## What exists
- `audio.rs`: AudioManager with volume settings and empty play_sound/play_music methods
- `client/assets/sounds/`: 389 ogg files organized by category
- `client/assets/sounds/SOUND_CATALOG.md`: full mapping of all sounds

## What needs to be built

### Sound loading
- On startup: scan client/assets/sounds/ directories
- Load all .ogg files into memory (rodio decoder)
- Map sound file names to sound IDs (match server's sound_id in SoundEffect packets)
- Create lookup table: sound_id -> decoded audio buffer
- Lazy loading option: load on first play for faster startup

### 3D positional audio
- Each sound has world position (from server S2CSoundEffect packet)
- Calculate relative position to camera/listener
- Volume falloff: linear or inverse-square, max 16 blocks hearing distance
- Stereo panning: based on angle between listener forward and sound direction
- Pitch variation: server sends pitch multiplier (0.5-2.0)

### Sound categories
- Master volume: multiplies all
- Music: background tracks
- Blocks: place, break, step sounds
- Hostile: mob sounds (zombie growl, skeleton rattle, creeper hiss)
- Friendly: passive mob sounds (cow moo, chicken cluck)
- Players: footsteps, damage, eating
- Ambient: environment loops (rain, water, lava, cave)
- Weather: rain, thunder
- Each category has independent volume slider in settings

### Block sounds
- On block break: play break sound for that material type
- On block place: play place sound
- Player footsteps: play step sound based on block below player, every ~0.5 sec while walking
- Material mapping: block_state_id -> material -> sound files

### Mob sounds
- Idle: random ambient sound every 5-15 sec
- Hurt: play on damage
- Death: play on kill
- Attack: play when mob attacks
- Special: creeper fuse sizzle, enderman scream, etc

### Music system
- Background music: random track every 10-30 min of silence
- Track selection: from client/assets/sounds/music/ folder
- Crossfade between tracks (2 sec fade)
- Music ducking: reduce music volume during combat
- Jukebox: play specific disc track when near active jukebox

### Ambient sounds
- Water: loop when near water blocks
- Lava: loop when near lava blocks (bubbling)
- Cave: random eerie sounds when in dark enclosed spaces
- Wind: loop when in void/on bridge, volume = wind strength
- Void whispers (ring 20+): random disorienting sounds near void edges

### Sky Craft specific sounds
- Wind gust: whoosh sound on gust warning
- Void lightning: crackling + thunder
- Aurora start/end: magical chime
- Debuff applied: distinct sound per debuff type
- Island tremor: rumbling bass
- Grappling hook: chain/rope sound + impact

### Performance
- Max simultaneous sounds: 32 (stop oldest if exceeded)
- Sound pooling: reuse stopped sound slots
- Distance culling: don't play sounds > 32 blocks away
- Background thread for audio mixing (rodio handles this)

### Files to create
- `client/src/audio/mod.rs` -- audio manager, init, tick
- `client/src/audio/loader.rs` -- ogg file loading, sound registry
- `client/src/audio/spatial.rs` -- 3D positional audio math
- `client/src/audio/music.rs` -- background music player
- `client/src/audio/ambient.rs` -- ambient sound loops

### Estimated: ~1500 lines
