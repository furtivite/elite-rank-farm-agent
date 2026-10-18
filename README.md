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

It is a single file: put it anywhere and double-click it. It needs the
Microsoft Visual C++ Redistributable (x64), which most games install anyway.
If Windows says `VCRUNTIME140.dll` is missing, install it from
<https://aka.ms/vs/17/release/vc_redist.x64.exe>.

> Windows may warn that the publisher is unknown: the build has no paid code
> signature. Click “More info” → “Run anyway”.

## Getting started

1. Sign in on the [site](https://frtvt.com/elite) and, in your account,
   in the agent section, press “Issue key”. The key is shown once — copy
   it.
2. Run `elite-agent.exe`. On the first start the settings are already open:
   paste the key into “Agent key” and press “Save”.
3. Press **“Session: on”** and play. The agent works while its window is
   open.

The window is in English. To switch it to Russian, open “⚙ Settings” and
choose “Русский” under “Language · Язык”; the choice is remembered.

## What the agent reads and where it sends it

Only what the site needs, and only from the game journal folder
(`Saved Games\Frontier Developments\Elite Dangerous`):

- **game journals** — your commander name; where you are (system, station,
  planet) and the systems you visited recently; your ship and what its jump
  range depends on (jump drive and booster, mass, fuel and cargo capacity);
  ranks and progress; fines with factions; accepted missions;
  how long your supercruise legs take; whether the game is running; and the
  stations, factions and game version you see;
- from **`Status.json`** — fuel and cargo in tons, because they change the
  jump range, and four flags (docked, landed, jump drive charging, low fuel)
  that only decide when to send — the flags themselves are not sent.
  Balance, legal status and the rest are not read;
- from **`Cargo.json`** — the total tonnage, and what is in the hold only to
  match it with your Collect and Mining missions. The site gets one number
  per such mission — how much of its commodity you carry — and never the
  list of your cargo.

It sends this **only to the project site**, over HTTPS and with your key.
Apart from its own folder below, the agent reads nothing else on the
computer and writes nowhere else. The one exception: on the first start of
version 0.6.2 or newer it moves its old files from next to the `.exe` into
that folder.

Its own files — settings, language, keys, the log and the queue of unsent data — live
in `%APPDATA%\elite-agent`. To remove the agent, delete the `.exe` and that
folder.

## Building from source

You need [Rust](https://rustup.rs) 1.88 or newer. The window builds on
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
