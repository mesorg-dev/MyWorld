# MyWorld

Prototype of a 2D top-down action arena built in Rust with macroquad.

## Run locally

1. Install the Rust toolchain if needed:
   - `curl https://sh.rustup.rs -sSf | sh -s -- -y`
   - `source $HOME/.cargo/env`
2. Start the game:
   - `cargo run`

## Controls

- Desktop: WASD to move, mouse to aim, left mouse button to strike, right mouse button to throw a knife, Left Shift to dash, Space to stop time, and F11 to toggle fullscreen.
- Android: use the left stick to move and the right stick only to aim. Hold Strike to attack while aiming, or use Knife, Dash, and Time. The touch controls support multiple fingers at once; tap the pause icon for the menu.
- Menu: use Up/Down and Enter or click/tap. Settings swaps the movement and aim controls for left-handed play. Press Esc to pause or return.

## Current implementation status

- Adaptive full-screen camera around a 480x270 arena with original neon pixel-art-style visuals
- WASD movement and mouse aiming
- Android multi-touch controls with a virtual movement stick and ability buttons
- Main menu, pause menu, and a left/right-handed touch layout setting
- Stand strike, knife throwing, and dash
- Cooldowns for dash, knife throws, and time stop
- Time stop lets the player move while enemies and knives freeze; attacks are queued until time resumes
- Distinct enemy movement, shooter and sniper projectiles, melee damage, health and time-charge bars, and repeating waves

## Release builds

GitHub Actions builds Windows and Linux binaries plus an Android APK on every branch push and uploads downloadable build artifacts. Push a version tag such as `v0.1.0` to publish those builds as a GitHub pre-release.
