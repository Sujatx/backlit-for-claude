# backlit-for-claude

Your ASUS TUF keyboard lights follow what Claude Code is doing.

| Claude Code is... | Keyboard |
|---|---|
| working | orange, breathing |
| waiting for you (permission, question) | red |
| done with its answer | green for 30 s |
| idle | colour cycle |
| hit an error or refusal | red |

It is a [Claude Code](https://claude.com/claude-code) mod: a few hooks plus a tiny Rust helper (`kbl.exe`) that sets the lighting. It uses no tokens, adds nothing to the model's context, and needs no admin rights.

## Tested on

| Laptop | BIOS | Result |
|---|---|---|
| ASUS TUF Gaming A15 FA506NCG (Windows 11) | FA506NCG.306 | Works: static, breathe and colour cycle |

Other TUF laptops that G-Helper drives the same way may work. They are untried; see Contributing.

## Install

In a terminal session:

```
/plugin install backlit --marketplace Sujatx/backlit-for-claude
```

Or from a shell:

```
claude plugin install backlit --marketplace Sujatx/backlit-for-claude
```

In the desktop app, where `/plugin` does not run: click **+** next to the prompt box, then **Plugins**, **Add marketplace** with `Sujatx/backlit-for-claude`, then **Add plugin**.

Choose the **user** scope to have it in every session.

## What it does to your laptop

`kbl.exe` opens the ASUS ACPI device (`\\.\ATKACPI`) and sends the same lighting commands [G-Helper](https://github.com/seerge/g-helper) sends: a colour and an effect for the keyboard. It never changes brightness, so your Fn-key level stays. This is an undocumented firmware interface, used at your own risk. Whether a setting survives sleep or a reboot has not been checked.

## Customise

- Colours and effects: edit the `state` table in `helper/src/main.rs` and rebuild (below). Modes are `0` static, `1` breathe, `2` colour cycle.
- Time green stays: `IDLE_AFTER_DONE_MS` in `hooks/register.ts`.
- Try a setting by hand: `bin\kbl.exe rgb <mode> <r> <g> <b> <speed>`.

## Build the helper yourself

The repo ships a prebuilt `bin/kbl.exe`. Check it, or replace it with your own build:

```
cd helper
cargo build --release
copy target\release\kbl.exe ..\bin\kbl.exe
```

SHA-256 of the shipped `bin/kbl.exe` (v0.1.1):

```
e52e55f787847f9fcaae63ef33a7b9aec35dcd4c177a4a6c477feeb2d0d8b49a
```

A rebuild may differ byte for byte, because Rust embeds paths and build details.

## Layout

```
.claude-plugin/   plugin and marketplace manifests
hooks/            the mod (register.ts) and its test
helper/           Rust source for kbl.exe
bin/              prebuilt kbl.exe
```

Run the test with `claude plugin test .`.

## Limits

- Several Claude Code sessions at once share one keyboard, so the last event wins.
- Windows only.

## Contributing

Help for other devices is very welcome. The most useful thing is a test report:

1. Install the mod, or just run `bin\kbl.exe rgb 0 255 0 0 0` (static red) in a terminal.
2. Open an issue with your laptop model, BIOS version (`Get-CimInstance Win32_BIOS`) and what happened to the keyboard, plus the `rgb=` number the command prints (`1` means the firmware accepted it).
3. Try modes `1` (breathe) and `2` (colour cycle) and tell us whether they matched.

I'll add working models to the table above. Pull requests are welcome too: support for other ASUS models or endpoints, per-key or multi-zone keyboards, other operating systems, other colours or states. Keep changes small, run `claude plugin test .` and say which hardware you tried it on. Please don't send untested firmware writes: stick to the documented G-Helper commands, and never use BIOS-persistent ones.

## Credits

The ACPI device path, request framing, device IDs and effect modes come from [G-Helper](https://github.com/seerge/g-helper) by seerge (`app/AsusACPI.cs`, GPL-3.0). This project reimplements that interface in Rust and does not include G-Helper's code. Thanks to the G-Helper authors and community.

## License

[MIT](LICENSE)
