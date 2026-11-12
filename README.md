# MyWorld

Prototype of a 2D top-down action arena built in Rust with macroquad.

## Run locally

1. Install the Rust toolchain if needed:
   - `curl https://sh.rustup.rs -sSf | sh -s -- -y`
   - `source $HOME/.cargo/env`
2. Start the game:
   - `cargo run`

## Current implementation status

- Letterboxed 480x270 arena with original neon pixel-art-style visuals
- WASD movement and mouse aiming
- Stand strike, knife throwing, and dash
- Time stop lets the player move while enemies and knives freeze; attacks are queued until time resumes
- Enemy melee damage, health and time-charge bars, and repeating waves

Controls: WASD move, mouse aim, left mouse strike, right mouse throw a knife, Left Shift dash, Space toggle time stop.

## Release builds

GitHub Actions builds Windows and Linux binaries on every branch push and uploads downloadable build artifacts. Push a version tag such as `v0.1.0` to publish those builds as a GitHub pre-release.
