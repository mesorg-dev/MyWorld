# MyWorld

Prototype of a 2D top-down action arena built in Rust with macroquad.

## Run locally

1. Install the Rust toolchain if needed:
   - `curl https://sh.rustup.rs -sSf | sh -s -- -y`
   - `source $HOME/.cargo/env`
2. Start the game:
   - `cargo run`

## Controls

- Desktop: WASD to move, mouse to aim, left mouse button to strike, right mouse button to throw a knife, Left Shift to dash, Space to stop time.
- Android: drag the left virtual stick to move and the right virtual stick to aim; while aiming, attacks repeat. Use the on-screen Knife, Dash, and Time buttons for abilities.

## Current implementation status

- Letterboxed 480x270 arena with original neon pixel-art-style visuals
- WASD movement and mouse aiming
- Android multi-touch controls with a virtual movement stick and ability buttons
- Stand strike, knife throwing, and dash
- Time stop lets the player move while enemies and knives freeze; attacks are queued until time resumes
- Enemy melee damage, health and time-charge bars, and repeating waves

## Release builds

GitHub Actions builds Windows and Linux binaries plus an Android APK on every branch push and uploads downloadable build artifacts. Push a version tag such as `v0.1.0` to publish those builds as a GitHub pre-release.
