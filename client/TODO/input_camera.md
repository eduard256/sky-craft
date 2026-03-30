# TODO: Input & Camera Control

## Status: Basic camera and input DONE, advanced features remaining

## What is DONE
- [x] InputState: keyboard + mouse tracking, per-frame pressed/clicked
- [x] Convenience methods: WASD, jump, sneak, sprint, hotbar keys, inventory, chat, F3
- [x] Camera struct: position, yaw, pitch, FOV, aspect, near/far
- [x] Mouse look: yaw/pitch from mouse delta with sensitivity
- [x] Camera movement: WASD + Space/Shift, sprint with Ctrl
- [x] View-projection matrix generation (glam)
- [x] Camera wired to input in state.rs update loop

## What still needs to be built

### Mouse capture
- Capture mouse on entering Playing state (hide cursor, lock to center)
- ESC: release mouse (show pause menu)
- Click to re-capture when window focused
- winit CursorGrabMode::Confined or Locked
- Raw mouse input (DeviceEvent::MouseMotion) for smoother look

### Player movement -> packets
- Send C2SPlayerPositionAndLook to server every tick when moving
- Client-side prediction: apply movement locally, server corrects
- Gravity + collision with local chunk data
- Sprint: double-tap W or hold Ctrl
- Sneak: hold Shift, prevent fall off block edges

### Action input -> packets
- Left click: raycast -> C2SBlockDig or C2SEntityInteract
- Right click: raycast -> C2SBlockPlace or C2SUseItem
- Scroll wheel: change hotbar slot (C2SHeldItemChange)
- Q: drop item, E: inventory, T: chat, F3: debug

### Block targeting (raycast)
- DDA ray-march from camera through voxel grid
- Max range: 5 blocks
- Return: block position + hit face + distance
- Render wireframe outline on targeted block

### Block breaking animation
- Crack overlay texture (10 stages) on targeted block face
- Progress based on tool + block hardness
- Cancel on target change or mouse release

### Files to create
- `client/src/renderer/outline.rs` -- block selection wireframe

### Estimated remaining: ~800 lines
