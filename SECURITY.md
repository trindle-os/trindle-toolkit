# Security policy

The Trindle Toolkit is looked after by one person, not a company or a team with a security desk. This page says what to expect without promising more than that.

## Supported versions

Only the latest release is supported. There is no long-term support branch and nothing is fixed in older versions: a fix lands in the next release and you are expected to move to it.

| Version | Supported |
|---|---|
| 0.9.0-beta.1 | yes |
| anything older | no |

## Reporting a vulnerability

**Please do not put the details of a security problem in a public issue.** A public report tells everyone, including people running an unfixed copy, before a fix exists.

Use GitHub's private vulnerability reporting on this repository: open **Security**, then **Report a vulnerability**. Only the maintainer can see it.

If you would rather not use GitHub, write to <toolkit@trindle.org> with "Toolkit security" in the subject line and say only that you have something to report. Leave the details out of that first message and a private way to share them will be arranged.

## What to expect

This is looked after in spare time, with no service level agreement. Best effort, not a guarantee:

- A reply to your report within a few days is the aim.
- A fix or a way round it depends on how serious and how tangled the problem is. It could be the same day for something simple and serious, or longer for something subtle.

## Scope

This covers the toolkit's own code, style sheet and language files. It does not cover:

- Faults in the libraries it uses. Report those to their own maintainers, or through [RustSec](https://rustsec.org/).
- A program built on the toolkit. Report that to the program's own maintainers.
- Your own computer: the compositor, the desktop and the fonts installed are outside this project's control.

## Known warnings

`cargo audit` lists three libraries as no longer maintained: `zvariant` (RUSTSEC-2026-0370), `zbus_names` (RUSTSEC-2026-0371) and `zvariant_derive` (RUSTSEC-2026-0372). They are warnings, not known faults. They come in through `zbus` and `accesskit`, whose newer versions need Rust 1.87, and the toolkit stays on Rust 1.85, the version Debian 13 ships. They will be replaced when the toolkit moves to a newer Rust.

## What is already in place

So a report can focus on what is new, [the Security section of the README](README.md#security) lists what is built in: a password box that is never read out, private files written whole, style sheets that are only read, strict rules for unsafe code and size limits on what is decoded.
