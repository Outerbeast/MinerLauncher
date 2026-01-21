# MinerLauncher

![alt text](https://github.com/Outerbeast/MinerLauncher/blob/main/preview.png?raw=true)

Quick launcher for GPU mining tools through a flightsheet system

MinerLauncher currently supports the following mining tools:
- rigel
- t-rex
- lolMiner
- xmrig
- wildrig
- gminer
- teamredminer
- PhoenixMiner
- srbminer
- bzminer
- nbminer

Each miner is mapped to the appropriate flags for:
- Algorithm
- Pool URL
- Wallet / Username
- Worker name
- Core clock
- Memory clock
- Fan speed
- Power limit


## Usage
1. 	Configure your flight sheet

Fill in the required fields in the GUI:
- Executable path (e.g., )
- Coin type
- Wallet address
- Worker name
- Pool URL and port
- Optional: core/memory clock, fan speed, power limit, extra arguments (requires admin privledges)

2. Manage flight sheets
- Use the dropdown to select a saved flight sheet.
- Click Save Sheet to store current settings.
- Click Add Sheet to create a new one.
- Click Clear to reset the form.

3. Start mining
- Click Start Miner to launch the miner with the configured settings


# Building from source

## Prerequisites

1️⃣ Install Rust

- Visit [https://rustup.rs](https://rustup.rs) and download the Windows installer.
- Run it and accept the defaults (this installs `cargo`, `rustc`, and `rustup`).
- Close and reopen any terminal/PowerShell windows after installation.

---

## Build instructions

1. Download or clone the repository:

```cmd
git clone https://github.com/Outerbeast/MinerLauncher.git
cd MinerLauncher
```

2. Run the build script:

- Double-click build.cmd or run it manually:

```cmd
build.cmd
```

The executable will be generated in the current directory.

# Credits
Outerbeast - Author


### Special Thanks

MinerLauncher user interface is powered by [Slint](https://slint.dev/).
