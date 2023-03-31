# minicrust

[minicraft](https://github.com/L3P3/minicraft), but written in [Rust](https://www.rust-lang.org/).

## Goals

- Learn and practice Rust
- Demonstrate ways to improve performance over JavaScript
- Show off being smarter than [LFF5644](https://github.com/LFF5644)
- Show off being faster than [Spickelbing](https://github.com/Spickelbing)

## Non-Goals

- Replace minicraft
- Replace Minecraft
- Render via GPU
- Ident code with spaces

## How to use

This app is not using fancy stuff like windows but the bare linux framebuffer device!

- Go into tty0 (ctrl+alt+f1)
- Make sure your user has access to the fb0 device (`sudo usermod -aG video $USER`)
- `./minicrust` or `cargo run` or whatever
- Press ctrl+c to exit
- Give this repo a star

## Special thanks to ...

- ChatGPT
- GitHub Copilot
- Google
