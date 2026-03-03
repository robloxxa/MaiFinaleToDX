# Maimai Finale to Maimai Deluxe wrapper

The most non-elegant way of making Maimai Deluxe playable on Maimai Finale Cabinet on Rust.

# Setup

## Requirements

- Windows (required — the tool uses Windows-only APIs)
- [com0com](https://sourceforge.net/projects/com0com/) or similar virtual COM port driver

## 1. Install com0com

Download and install [com0com](https://sourceforge.net/projects/com0com/). It creates pairs of virtual COM ports connected to each other — whatever is written to one end is read from the other.

## 2. Create virtual COM port pairs

You need two pairs for the touch screen (P1 and P2). Use the com0com setup utility to create them.

Default config expects the wrapper to write to these ports:

| Purpose | Wrapper writes to | Game reads from |
|---------|-------------------|-----------------|
| Touch P1 | `COM6` | paired port (e.g. `COM7`) |
| Touch P2 | `COM8` | paired port (e.g. `COM9`) |

Create the pairs in com0com: `COM6 <-> COM7` and `COM8 <-> COM9`. Configure the DX game to read touch from `COM7` and `COM8` (or whichever ports are on the game's side of the pairs).

If using a hardware NFC card reader, create an additional pair for it (default: `COM24`).

## 3. Generate the config file

Run the executable with the `-c` flag to create a default `config.toml`:

```bash
MaiFinaleToDX.exe -c
```

## 4. Edit config.toml

Open `config.toml` and adjust the COM port settings to match your setup:

```toml
[touch.finale]
enabled = true
port = "COM23"        # COM port of the Finale touch screen (real hardware)

[touch.dx]
enabled = true
p1_port = "COM6"      # Virtual port for P1 touch (game reads from the paired port)
p2_port = "COM8"      # Virtual port for P2 touch (game reads from the paired port)

[jvs]
enabled = true
port = "COM23"        # COM port of the Finale JVS I/O board

[reader]
enabled = false
port = "COM24"        # COM port of the Finale NFC card reader
```

Set `enabled = false` for any module you are not using.

## 5. Run

```bash
MaiFinaleToDX.exe
```

The GUI window will open showing the status of each module. To run without the GUI:

```bash
MaiFinaleToDX.exe --no-gui
```

Logs are saved to `./logs/` by default.


# About

## How does it work?
It takes inputs from Cabinet Touchscreen ~~/JVS/NFC reader~~ COM Ports, modifies it in the way that Deluxe can understand
and sends it to other virtual COM ports (via [com0com]() or other programs) that game will read.
## Touchscreen difference
If you played both Finale and Deluxe version, you know that Deluxe have new additional touch zones as well as a new game mechanic - touch notes.

Here is the difference between Finale (on the left) and Deluxe (on the right) 

![](https://i.imgur.com/w8sUFHy.png)

Since the layout is pretty much the same, we simply mapped new zones to the closest existing ones. 
E.g., if you press B1, the E1 and E2 will also activate.
Same with A and D zones.

## Why keyboard emulation with JVS?

I just didn't figure out how to make Deluxe read from JVS COM port. 

In theory, it uses a combination of `COM4` and `\\.\mxjvs`, but the game never send anything to these ports.

If you know how to solve this, please make a PR or DM me on [Discord](https://discordapp.com/users/161178211596763137)


# Build
1. Install Rust via [rustup](https://rustup.rs/) or via [other methods](https://forge.rust-lang.org/infra/other-installation-methods.html)
   1. If you're building on Mac or Linux, install `stable-x86_64-pc-windows-gnu` toolchain and change it via `rustup default stable-x86_64-pc-windows-gnu` command
2. Clone this repository 
    ```bash
    git clone https://github.com/robloxxa/MaiFinaleToDX.git
    cd MaiFinaleToDX
    ```
3. Run `cargo build --release`

# Configuration Reference

Config is loaded from `./config.toml` by default. Generate a template with `MaiFinaleToDX.exe -c`.

### `mode` field

All three modules (`touch.finale`, `touch.dx`, `jvs`, `reader`) share a `mode` field:

| Value | Description |
|-------|-------------|
| `"Hardware"` | (default) Read from / write to a real COM port |
| `"Emulated"` | Spawn an internal software emulator instead of using real hardware. Useful for testing without a cabinet. |

---

## `[touch.finale]`

Reads touch input from the Finale cabinet's touchscreen.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | `true` | Enable this module |
| `mode` | string | `"Hardware"` | See [mode](#mode-field) |
| `port` | string | `"COM23"` | COM port of the Finale touch controller |
| `init_retry_count` | integer \| null | `null` | How many times to retry initialization before giving up. `null` = retry forever |

### `[touch.finale.p1_threshold]` / `[touch.finale.p2_threshold]`

Capacitive touch sensitivity thresholds for each zone. Higher value = less sensitive. Zones are `A1`–`A8`, `B1`–`B8`, and `C`.

```toml
[touch.finale.p1_threshold]
A1 = 65
A2 = 130
A3 = 200
A4 = 180
A5 = 180
A6 = 200
A7 = 130
A8 = 65
B1 = 100
B2 = 100
B3 = 170
B4 = 140
B5 = 140
B6 = 170
B7 = 100
B8 = 100
C  = 110
```

---

## `[touch.dx]`

Converts Finale touch input to DX format and writes it to the virtual COM ports the game reads from.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | `true` | Enable this module |
| `mode` | string | `"Hardware"` | See [mode](#mode-field) |
| `p1_port` | string \| null | `"COM6"` | COM port for P1 touch output |
| `p2_port` | string \| null | `"COM8"` | COM port for P2 touch output |

### `[touch.dx.p1_mapping]` / `[touch.dx.p2_mapping]`

Defines which Finale zones activate each DX zone. Each DX zone entry has three fields:

| Field | Type | Description |
|-------|------|-------------|
| `activate_on` | string[] | List of Finale zone names (`"A1"`–`"A8"`, `"B1"`–`"B8"`, `"C"`) that trigger this DX zone |
| `deactivate_after_ms` | integer | Force-deactivate the zone after N milliseconds. `0` = disabled |
| `reactivate_after_ms` | integer | Prevent re-activation for N milliseconds after deactivation. `0` = disabled |

DX zones: `A1`–`A8`, `B1`–`B8`, `C1`, `C2`, `D1`–`D8`, `E1`–`E8`.

The default mapping mirrors the zone layout described in [Touchscreen difference](#touchscreen-difference): D and E zones (which only exist in DX) are mapped to the nearest Finale A and B zones respectively.

---

## `[jvs]`

Reads button input from the Finale JVS I/O board and re-sends it as keyboard key presses to the game.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | `true` | Enable this module |
| `mode` | string | `"Hardware"` | See [mode](#mode-field) |
| `port` | string | `"COM23"` | COM port of the JVS I/O board |
| `init_retry_count` | integer \| null | `null` | How many times to retry initialization. `null` = retry forever |

### `[jvs.input]`

Maps JVS buttons to [Windows Virtual Key codes](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes) (decimal integers).

| Key | Default | Default key |
|-----|---------|-------------|
| `test` | `84` | T |
| `service` | `51` | 3 |
| `p1_btn1` | `87` | W |
| `p1_btn2` | `69` | E |
| `p1_btn3` | `68` | D |
| `p1_btn4` | `67` | C |
| `p1_btn5` | `88` | X |
| `p1_btn6` | `90` | Z |
| `p1_btn7` | `65` | A |
| `p1_btn8` | `81` | Q |
| `p2_btn1` | `104` | Numpad 8 |
| `p2_btn2` | `105` | Numpad 9 |
| `p2_btn3` | `102` | Numpad 6 |
| `p2_btn4` | `99` | Numpad 3 |
| `p2_btn5` | `98` | Numpad 2 |
| `p2_btn6` | `97` | Numpad 1 |
| `p2_btn7` | `100` | Numpad 4 |
| `p2_btn8` | `103` | Numpad 7 |

---

## `[reader]`

Bridges the Finale NFC card reader to the DX game.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enabled` | bool | `false` | Enable this module |
| `mode` | string | `"Hardware"` | See [mode](#mode-field) |
| `port` | string | `"COM24"` | COM port of the NFC card reader |
| `device_file` | string \| null | `"./device.txt"` | In `Emulated` mode: path to a text file containing a card ID to simulate |
| `destinations` | integer[] | `[0, 1]` | Reader destination addresses. Finale cabinets have two readers at `0` and `1`. Do not change unless you know what you are doing. Max 4 entries. |
| `init_retry_count` | integer \| null | `null` | How many times to retry initialization. `null` = retry forever |
