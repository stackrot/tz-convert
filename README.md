# tz-convert

Convert times between any of the IANA timezones bundled with `chrono-tz`.
This includes UTC, regional names such as `Africa/Nairobi` and
`Pacific/Chatham`, and aliases such as `US/Eastern`.

Running `tz-convert` without arguments shows local time and UTC. In a terminal,
it then offers searchable source and destination pickers. Type a timezone or
city name to search, press Enter to select, and add as many destinations as you
need. Escape cancels a picker. Invalid dates are explained and can be corrected.

## Download

Download and extract an archive from the [latest release](https://github.com/stackrot/tz-convert/releases/latest):

| Platform | Download | Requirements |
| --- | --- | --- |
| Linux | [tz-convert-linux.zip](https://github.com/stackrot/tz-convert/releases/latest/download/tz-convert-linux.zip) | x86-64, glibc 2.35 or newer |
| Windows | [tz-convert-windows.zip](https://github.com/stackrot/tz-convert/releases/latest/download/tz-convert-windows.zip) | x86-64 |

On Linux:

```sh
unzip tz-convert-linux.zip
chmod +x tz-convert
./tz-convert
```

On Windows, extract the ZIP and run `tz-convert.exe`. From PowerShell:

```powershell
Expand-Archive tz-convert-windows.zip
.\tz-convert-windows\tz-convert.exe
```

Each new release includes `SHA256SUMS` for verifying the downloaded archives.

## Command-line usage

List every supported timezone:

```sh
tz-convert --list-timezones
```

Convert a London time to New York and Kathmandu:

```sh
tz-convert --time "2026-07-15 12:00" --from Europe/London --to America/New_York,Asia/Kathmandu
```

`--to` can also be repeated. The source timezone is always displayed alongside
the destinations, and duplicate destinations appear only once.

Show the current time in selected zones, or in every zone:

```sh
tz-convert --to Africa/Nairobi --to Pacific/Chatham
tz-convert --all
```

Convert a time to every zone:

```sh
tz-convert --time "2026-12-31 23:30" --from UTC --all
```

`--from` defaults to `local`, which uses the system timezone. `local` can also be
used as a destination. Named timezones are case-sensitive; use
`--list-timezones` for their exact names. With no destinations, the output shows
the source timezone and UTC. Omit `--time` to use the current instant.

Explicit command-line options never prompt. With no arguments and redirected
input or output, the program prints local time and UTC and exits. Invalid input
returns a non-zero exit status. Use `--help` for all options.

## Dates and daylight saving

Enter dates as `YYYY-MM-DD HH:MM` in the source timezone. Output includes the
date, timezone abbreviation and numeric UTC offset, including fractional-hour
and historical second-level offsets.

Times skipped by a clock change are rejected. Times that occur twice when
clocks go back are also rejected, with both possible offsets shown. To convert
one of those occurrences, supply its unambiguous UTC time with `--from UTC`.

Supported input years are **1800–2099**, matching the range of recurring rules
expanded by the bundled timezone library. Dates outside that range are rejected
rather than silently using an incorrect offset. Future conversions use the
rules in the bundled database; timezone rule changes require an updated build.

## Build and checks

Install stable Rust, then:

```sh
cargo build --release --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

The binary is `target/release/tz-convert` on Linux and
`target/release/tz-convert.exe` on Windows. `Cargo.lock` records the dependency
versions used by builds.

CI checks formatting, Clippy and tests on Linux and Windows, and checks the
lockfile against RustSec advisories. Successful pushes to `main` publish both
archives and their checksums.

Report bugs on the [Issues page](https://github.com/stackrot/tz-convert/issues).
