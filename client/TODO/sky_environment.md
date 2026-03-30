# TODO: Sky & Environment Rendering

## Status: Not implemented (clear color only)

## What exists
- `renderer.rs`: clear color changes by AppState (blue for Playing)
- `world.rs`: weather, time_of_day, wind state tracked

## What needs to be built

### Sky rendering
- Skybox or sky dome rendered behind everything (furthest depth)
- Sky color gradient: changes with time of day
  - Sunrise (0-1000 ticks): dark blue -> orange -> light blue
  - Day (1000-12000): light blue top, white-blue horizon
  - Sunset (12000-13000): light blue -> orange/red -> dark blue
  - Night (13000-23000): dark blue/black
- Ring darkening: reduce sky brightness at high rings (see unique_logic/hazards.md)

### Sun and moon
- Sun: textured quad on sky dome, rotates with time_of_day
- Moon: opposite side from sun, 8 phase textures (cycle every 8 days)
- Sun/moon scale: large enough to be visually prominent
- Textures from client/assets/textures/minecraft/textures/environment/

### Stars
- Visible at night (time 13000-23000)
- Random dot positions on sky dome (generated once from seed)
- Fade in at sunset, fade out at sunrise
- Twinkle effect: slight brightness variation

### Clouds
- Flat layer of cloud blocks at fixed Y height (above islands)
- Slowly drift in one direction
- Semi-transparent white
- Render as large textured quad or generated from noise
- Can be toggled in settings (Fancy clouds / Fast clouds / Off)

### Fog
- Distance fog: blend to sky color at render distance edge
- Smooths out chunk loading boundary
- Fog color matches sky/horizon color
- Underwater: blue fog at short distance
- Void fog (Sky Craft): gray/dark fog during void fog event
  - Triggered by server HazardWarning packet
  - Reduce effective render distance
  - Fog color: dark gray

### Weather rendering
- Rain: falling particle streaks, everywhere in loaded chunks
  - Blue-gray translucent lines falling from sky
  - Splash particles on block surfaces
  - Darken sky, reduce sky light
  - Rain sound ambient loop
- Thunder: lightning flash (screen white flash for 1 frame)
  - Lightning bolt: bright white line from sky to ground
  - Thunder sound after delay
- Snow: white particle dots falling slowly (in snowy biomes or high altitude)

### Aurora (Sky Craft unique)
- Triggered by S2CAuroraEvent packet
- Render colored light ribbons across sky
- Animated: flowing green/blue/purple bands
- Vertex shader: sin wave displacement for flowing effect
- Fades in over 5 sec, fades out over 5 sec before end

### Void rendering
- Below islands: dark gradient fading to deep blue/black
- Faint floating particles in void (sparse white/blue dots)
- Deeper = darker
- Waterfall off island edges: animated water stream fading downward
- Wind particles: horizontal streaks in wind direction (density = wind strength)
- Void well particles: swirling dark vortex (if near void well)
- Updraft particles: swirling white column upward

### Day/night lighting
- Sky light affects block face brightness
- Day: full brightness on exposed surfaces
- Night: dim, blue tint on moonlit surfaces
- Indoor: only block light (torches etc)
- Smooth transition between day/night

### Files to create
- `client/src/renderer/sky.rs` -- sky dome, sun, moon, stars
- `client/src/renderer/clouds.rs` -- cloud layer
- `client/src/renderer/weather.rs` -- rain, snow, lightning
- `client/src/renderer/fog.rs` -- distance fog, void fog
- `client/src/renderer/aurora.rs` -- aurora event rendering
- `client/src/renderer/void.rs` -- void visual effects, particles

### Estimated: ~2000 lines
