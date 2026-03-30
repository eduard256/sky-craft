# TODO: Input & Camera Control

## Status: Partial (key/mouse tracking exists, no camera or action dispatch)

## What exists
- `input.rs`: InputState tracks keys, mouse, scroll, convenience methods
- No camera struct, no mouse capture, no input -> packet conversion

## What needs to be built

### Mouse capture
- On entering Playing state: capture mouse (hide cursor, lock to center)
- ESC: release mouse (pause menu)
- Click in window when not captured: re-capture
- winit CursorGrabMode::Confined or Locked
- Raw mouse input for look (DeviceEvent::MouseMotion preferred)

### Camera control
- Yaw: mouse X movement * sensitivity
- Pitch: mouse Y movement * sensitivity, clamped -89..+89 degrees
- Sensitivity setting: configurable in settings (default 0.15 deg/pixel)
- Invert Y option

### Player movement input -> packets
- Every tick (50ms): if position/look changed, send to server
- Build C2SPlayerPositionAndLook from current input state
- Movement prediction: move locally, server corrects if wrong
  - Apply WASD input to velocity based on look direction
  - Apply gravity, collision with local chunk data
  - Show predicted position immediately, correct on server response
- Sprint: double-tap W or hold Ctrl
- Sneak: hold Shift, prevent fall off edges

### Action input -> packets
- Left click: C2SBlockDig (start/finish digging) or C2SEntityInteract (attack)
  - Raycast from camera to determine what's being looked at
  - If block: start digging animation, send Start, track progress, send Finish
  - If entity: send Attack
- Right click: C2SBlockPlace or C2SUseItem
  - If looking at block: place block on face
  - If holding usable item (food, bow): use item
- Scroll wheel: change held hotbar slot
- Q: drop item
- E: toggle inventory screen
- T: open chat
- F3: toggle debug screen
- 1-9: select hotbar slot

### Block targeting (raycast)
- Cast ray from camera position in look direction
- Step through blocks (DDA algorithm or similar)
- Max range: 5 blocks (survival), 5 blocks (creative can be extended)
- Return: hit block position + face hit + distance
- Highlight targeted block: render wireframe outline around it
- Show block face indicator (which face will be placed against)

### Block breaking animation
- On start digging: show crack overlay on targeted block face
- 10 stages of cracking texture (from MC textures)
- Progress rate depends on tool + block hardness (from server or local calc)
- Cancel if target changes or mouse released

### Hotbar scrolling
- Mouse scroll: change held slot (wrap 0-8)
- Number keys 1-9: direct slot selection
- Send C2SHeldItemChange to server on change

### Files to create/modify
- `client/src/input.rs`: add mouse capture, sensitivity, action dispatch
- `client/src/renderer/camera.rs`: camera struct, view/proj matrices, raycast
- `client/src/state.rs`: wire input to network packet sending
- `client/src/renderer/outline.rs`: block selection wireframe

### Estimated: ~1500 lines
