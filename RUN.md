# How to Run the CHIP-8 Emulator

Follow the steps below to download and run the project from scratch.
These instructions assume you have Rust and Cargo installed. If you don’t, you can install them here: https://rustup.rs

1. Use git clone to download the project to your machine: git clone https://github.com/VaibhavKGoel/Chip-8-Emulator
2. Move into the project folder: cd Chip-8-Emulator
3. Move into the Desktop Project Folder: cd ./desktop
4. Use cargo run and provide the path to a CHIP-8 ROM file: cargo run ./c8games/GAME
* Replace GAME with the filename of the CHIP-8 game you want to run (for example: PONG, TETRIS, INVADERS, etc.).
* All available test ROMs/games are located inside the c8games/ folder.
