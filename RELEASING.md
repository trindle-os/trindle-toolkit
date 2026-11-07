# Releasing

Nothing is published until the project says Trindle is complete. This is the order to follow when it does.

## Versions

- `trindle-common` and `trindle-toolkit` carry the same version (now `0.9.0-beta.1`), and the toolkit's dependency on `trindle-common` names that version beside its path.
- Change all three places together: `Cargo.toml` (`version`, and `trindle-common`'s `version` in `[dependencies]` and `[build-dependencies]`), `trindle-common/Cargo.toml`, then run `cargo +1.85 build --release` so `Cargo.lock` follows.
- Add a heading with the version and date in `CHANGELOG.md` and move what is under Unreleased beneath it. Keep `debian/changelog` in step.

## Before publishing

1. Work from a clean checkout of the release commit: `git status` shows nothing, and no task list or question notes are in the folder.
2. `cargo +1.85 test --release --workspace` passes, and so does `cargo +1.85 test --release --doc --features ui` (the guides in `docs/guides` are doctests).
3. `cargo +1.85 clippy --release --workspace --all-targets --features ui,testing` has zero warnings.
4. `cargo +1.85 build --release --examples` builds.
5. `trindle-lang-check` exits 0; `trindle-lang-check --missing` shows how far each language is.
6. The look tests pass: `every_part_in_every_state_is_drawn_and_none_overflows`, the stored-picture test and the speed budget test. If a look was changed on purpose, refresh the pictures with `TRINDLE_GOLDEN=refresh cargo +1.85 test --release parts_look_as`, look at `tests/golden/*.png`, and commit them with the change.
7. Public code goes through the publish strip first; push only its comment-free copy to GitHub.
8. The README's Building section, widget table and examples still match the code.

## Packaging

1. `cargo +1.85 package --list -p trindle-common` and `cargo +1.85 package --list` show only what should ship: source, `share/`, `docs/`, `examples/`, `protocols/`, `tests/golden`, the licence and the changelog, and no notes files. (`--allow-dirty` is only for looking on a working copy.)
2. Publish `trindle-common` first: `cargo +1.85 publish -p trindle-common --dry-run`, then the real one when the project says so.
3. Then the toolkit: `cargo +1.85 publish --dry-run`, then the real one. The toolkit's dry run needs `trindle-common` to be on the registry already.
4. `tools/lang-deb.sh [folder]` builds `trindle-toolkit-lang_VERSION_all.deb` (default `target/deb`), which puts the language files at `/usr/share/trindle-toolkit/lang`; Trindle and Folders depend on it.
5. `tools/release-build.sh` builds the toolkit's programs for shipping into `target/shipped/release`, apart from everyday builds, with no path from the build computer in them: the home folder, the folder the checkouts sit in, cargo's own folder and the target folder become `/home`, `/src`, `/cargo` and `/target`, and `--cfg trindle_shipped` leaves out `trindle-lang-check`'s look in cargo's fetched copy. A program built on the toolkit does the same with `RUSTFLAGS="--remap-path-prefix=$HOME=/home --remap-path-prefix=<folder of its checkout's parent>=/src --remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo --remap-path-prefix=<its target folder>=/target"`, in that order.
6. Tag the commit `v<version>` and make the GitHub release from the changelog's text.

## After

Tell the Trindle chat the version is out, so Trindle's workspace can point at it in place of the copy under `crates/`.
