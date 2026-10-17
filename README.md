# Elite Rank Farm Agent

**English** · [Русский](README.ru.md)

The companion agent for **[Elite Dangerous Rank Farm Navigator](https://frtvt.com/elite)**
— a site that picks stations for farming Imperial and Federal Navy ranks.

The agent is a small Windows program. While you play, it reads the game
journal and tells the site where you are, which ship you fly, how many
missions you have and what your ranks are. The site uses that to work out
where to fly next.

**Site: <https://frtvt.com/elite>**

## Download

**[Download the latest version](https://github.com/furtivite/elite-rank-farm-agent/releases/latest)**
— the release page has `elite-agent.exe` under “Assets” and the list of
changes in that version. All versions are on the
[releases](https://github.com/furtivite/elite-rank-farm-agent/releases) page,
the full history is in the [changelog](agent/CHANGELOG.md).

Nothing to install: it is a single file. Put it anywhere and double-click it.

> Windows may warn that the publisher is unknown: the build has no paid code
> signature. Click “More info” → “Run anyway”.

## Getting started

The agent window is in Russian for now; the button names are given below.

1. Sign in on the [site](https://frtvt.com/elite) and, in your account,
   in the agent section, press “Issue key” («Выпустить ключ»). The key is
   shown once — copy it.
2. Run `elite-agent.exe`. On the first start the settings are already open:
   paste the key into “Agent key” («Ключ агента») and press “Save”
   («Сохранить»).
3. Press **“Session: on”** («Сессия: включить») and play. The agent works
   while its window is open.

## What the agent reads and where it sends it

Only what the site needs, and only from the game journal folder
(`Saved Games\Frontier Developments\Elite Dangerous`):

- **game journals** — your location, ship and its jump range, ranks and
  progress, accepted missions, station and faction details;
- from **`Status.json`** — only fuel and cargo in tons: they change the jump
  range. Balance, legal status and the rest are not read;
- from **`Cargo.json`** — only the total tonnage, not what the cargo is.

It sends this **only to the project site**, over HTTPS and with your key.
The agent reads nothing else on the computer and writes nowhere else.

Its own files — settings, keys, the log and the queue of unsent data — live
in `%APPDATA%\elite-agent`. To remove the agent, delete the `.exe` and that
folder.

## Building from source

You need [Rust](https://rustup.rs) 1.82 or newer. The window builds on
Windows only; journal parsing and sending build on any OS.

```text
cd agent
cargo build --release
```

The result is `agent/target/release/elite-agent.exe`. `contracts/` holds
the data schemas the agent exchanges with the site.

## License

[Apache License 2.0](LICENSE).

## Legal notice

Unofficial fan project. Not affiliated with or endorsed by Frontier
Developments plc. Elite Dangerous and related materials are the property of
their respective owners. The agent icon was drawn for this project; no game
logos are used.
