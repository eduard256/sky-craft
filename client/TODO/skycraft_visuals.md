# TODO: Sky Craft Visual Effects

## Status: Not implemented

## What exists
- `world.rs`: wind, ring, debuffs, island_info tracked from server
- Protocol: all Sky Craft visual packets defined

## What needs to be built

### Wind visual effects
- Horizontal particle streaks in wind direction
- Density proportional to wind strength
- White/gray semi-transparent streaks
- Only visible when in open air (between islands, on bridges)
- Gust: sudden burst of dense particles for 1-3 sec

### Void visual effects
- Below islands: dark gradient fading to deep blue/black
- Sparse floating particles (small white/blue dots drifting upward)
- Deeper void = fewer/dimmer particles
- Waterfall from island edges: animated water texture stream fading down ~20 blocks
- Lava-fall: orange/red stream, shorter fade

### Debuff visual indicators
- Screen overlay effects when debuffs active:
  - Placement Lock: faint red border flash
  - Mining Lock: faint orange border flash
  - Fear: screen shake + dark vignette
  - Void Sickness: green tint + slight wobble (nausea)
  - Gravity Pull: directional pull lines on screen edges
- Debuff icon rendering on HUD (left side, below hearts)

### Ring transition effect
- When crossing ring boundary: brief screen-wide text notification
- "Entering Ring N" with ring-appropriate color
- Subtle color shift in sky tint for 2-3 seconds
- Sound cue: deep tone that rises with ring number

### Aurora rendering
- Colored light ribbons across sky
- Animated: vertex displacement with sin waves
- Colors: green base + blue + purple highlights
- Fades in over 5 sec when S2CAuroraEvent(active=true) received
- Fades out over 5 sec before ending
- Everything gets slightly brighter ambient light during aurora

### Void lightning
- On S2CHazardWarning(VoidLightning): flash warning icon 1 sec before
- Lightning bolt: bright purple-white line from sky to target position
- Screen flash: brief white overlay (100ms)
- Illuminate nearby void for 0.5 sec (show island undersides)
- Destroyed bridge blocks: particle explosion at impact point

### Cloud platforms
- Semi-transparent white block-like platforms in void
- Slightly translucent, wobble animation
- When player stands on: slowly become more transparent (sinking visual)
- When player falls through: dissolve particle effect

### Updraft visual
- Swirling white particle column (3-5 blocks wide)
- Particles spiral upward
- Visible from distance (draw as billboard or particle system)
- Sound: rushing air loop while near

### Void well visual
- Dark swirling particle vortex
- Particles pulled inward and downward
- Dark purple/black tint
- Distortion effect on nearby geometry (optional, shader-based)
- Sound: deep humming

### Island magnetism visual
- Faint shimmering particle effect at island edges
- Very subtle, only visible within 3 blocks of edge
- Indicates the slow-fall zone

### Resonance mining glow
- When player hits ore: nearby same-type ores glow through stone
- Bright colored outline around ore blocks
- Visible through solid blocks (depth test disabled for glow)
- Duration: 3 seconds, fade out over last 0.5 sec
- Color matches ore type (blue for diamond, red for redstone, etc)

### Grappling hook visual
- Chain/rope line from player to hook point
- Segmented line with slight sag (catenary curve)
- Animate player flying along the line
- Impact particles when hook hits block

### Sky lantern
- Small glowing quad floating upward
- Warm orange light, gentle sway
- Light level 12 illuminating nearby blocks
- Fade out when timer expires (5 min)

### Files to create
- `client/src/renderer/effects/mod.rs` -- effect dispatcher
- `client/src/renderer/effects/wind.rs` -- wind particles
- `client/src/renderer/effects/void.rs` -- void visuals, waterfalls
- `client/src/renderer/effects/debuff.rs` -- screen overlays for debuffs
- `client/src/renderer/effects/aurora.rs` -- aurora sky ribbons
- `client/src/renderer/effects/lightning.rs` -- void lightning bolt
- `client/src/renderer/effects/skycraft.rs` -- cloud platforms, updrafts, wells, etc

### Estimated: ~2500 lines
