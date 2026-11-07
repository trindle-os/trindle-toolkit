# Trindle Toolkit measurements, 28 Oct 2026

Machine: 12 cores, Rust 1.85, release build, software drawing (tiny-skia). Test: `cargo +1.85 test --release --lib measure_the_gallery -- --ignored --nocapture` (src/gallery.rs). The page is every control at 1280 x 800.

| What | Result |
|---|---|
| First page (build, lay out, paint, fonts cold) | 196 ms |
| Paint again, nothing changed | 10.3 ms (median of 200) |
| Pointer move, then paint | 10.3 ms |
| New view, lay out and paint | 23.4 ms |
| Memory taken by the page | 14.9 MB (test process 20.8 MB) |
| Hello example, binary size | 14.9 MB (unstripped release) |
| Hello example, shared libraries | 1 outside the C library (libxkbcommon) |

Not measured: GTK 4 (installed, 4.18), GTK 3, Qt (not installed), iced 0.14 (source in the cargo cache, not built), libcosmic. No claim is made against them until the same page is built in each and timed the same way.

## What the numbers say
- Every repaint redraws the whole page (10 ms, so a 60 Hz frame budget of 16.7 ms is 60 per cent used by a still page). A toolkit that repaints only what changed would do far better. This is the first thing to fix before claiming speed.
- The 196 ms first page is mostly font loading; worth splitting out.

## After the first fixes (same day)
The machine is shared with other sessions, so every figure moves by half again between runs; compare only figures from one run.

- Shadows (`src/shadow.rs`): laying a shadow worked out the same index sums for every pixel and tested each pixel against the box. It now builds the column table once and skips the box's inside by range. Same pixels, same tests. Whole-page paint fell from 6.5 to 4.8 ms in one run.
- Pointing at another widget used to paint the whole window. Now `Reply::patch` and `Ui::touch` paint only the two widgets (and the room a shadow or lit edge needs) and the window puts just those parts on screen. In one run: whole 3.57 ms, the two parts 0.50 ms (7 times less).
- Test `pointing_at_one_widget_after_another_paints_only_them_and_changes_nothing_else` checks the patched window equals a whole painting to within 6 of 255 levels (rounded corners of cards that run off a part are clipped differently by tiny-skia); it fails on any real difference.
- Not yet covered: a press, a key, a typed letter and a scroll still paint the whole window. The next weakest figure is the whole paint itself (cards cost most, about half).
- Not run on a live compositor here; the window code that puts parts on screen is the same as for spinners.

## Processor-only drawing (virtual machines with no graphics card)
(Written on 28 Oct, before the GPU drawer; see the final figures at the end.) Everything this toolkit then drew was drawn by the processor (tiny-skia). `TRINDLE_DRAWN_BY_PROCESSOR=1` makes it cheaper:
- soft shadows are left out (cards keep their edge line), and a spinner, glow or bell steps every 32 ms instead of 16 ms.

Same page, same run each (1280 x 800, `measure_the_gallery`):

| | normal | =1 |
|---|---|---|
| Whole page paint | 3.68 ms | 1.78 ms |
| Pointer move to another widget, whole | 3.56 ms | 1.67 ms |
| Pointer move, only the two painted | 0.49 ms | 0.35 ms |
| New view, lay out and paint | 8.19 ms | 6.10 ms |
| Whole page at half opacity (layer laid over) | 3.45 ms | 1.60 ms |
| First page (fonts cold, noisy) | 118 ms | 29 ms |

Shadows are about half of a whole paint. A half-opacity layer adds almost nothing over a plain paint. Swapping the 4 MB frame into the compositor's colour order takes 0.46 ms.

Copies of the frame for one whole paint: clear to zero (4 MB), paint, swap red and blue (4 MB), the compositor reads it. A part patch copies only the part, plus one whole-frame copy when the other buffer is behind. Idle: nothing is scheduled when no spinner, glow, bell, name or armed button is showing (no wakeups per second); one spinner wakes 62 times a second, 31 with the variable on.

## Damage tracking, state at the end of 28 Oct
- Done: pointing at another widget and a button going down repaint only the widgets that changed (about 0.5 ms against 3.6 ms for the page, 1.7 against 0.35 with TRINDLE_DRAWN_BY_PROCESSOR=1). A window that nothing changed is not repainted at all (nothing asks for it), so a still window costs no frames.
- Still whole-window: any message that reaches the program (a button let go, text typed, a slider moved), scrolling, a name appearing, dropdowns. These need the program's new view compared with the old one, widget by widget, which the tree does not yet support; that is the next step.

## First window and memory, against GTK 4 (28 Oct, this machine, real Wayland session)
The 196 ms first-page figure above was noise from other sessions loading the machine. Quiet runs of `measure_the_first_page`: fonts found 8 ms, Ui::new 8 ms, first lay out 5 ms, first paint 20 ms (glyph shapes made once), second paint 3.7 ms; about 42 ms in all.

The same small window (a line of words and one button, 320 x 160) in each, timed with `WAYLAND_DEBUG=1` from the program's first `get_registry` to its first `wl_surface.attach` of a buffer, and sized by `VmRSS` after 1.5 s:

| | Trindle Toolkit `hello` | GTK 4.18 (Python, GL) |
|---|---|---|
| Registry to first buffer | 23, 28 and 35 ms | 575, 595 and 924 ms |
| Memory resident | 14.8 MB (whole process) | 145 MB more than before the window (190 MB whole process) |

GTK was driven from Python and drew with Mesa GL, so its figures include that stack; a C program would shave some, not 20 times. Not measured: Qt (not installed), Iced 0.14 (does not build here: its softbuffer dependency fails to compile on Rust 1.95), Cosmic (libcosmic not present). No claim is made about them.

## Against Qt, Iced and GTK 4 (same small window, 28 Oct, this machine)
Method as above: `WAYLAND_DEBUG=1`, first `get_registry` to first `wl_surface.attach` of a buffer, five runs for the toolkit; resident memory after 1.5 to 3 s.

| | Registry to first buffer | Memory resident | Binary (stripped) | How it was run |
|---|---|---|---|---|
| Trindle Toolkit, before the fix below | 23, 28, 35 ms | 14.8 MB | 12.1 MB | Rust, tiny-skia |
| Trindle Toolkit, after the fix | 15 to 18 ms | 14.8 MB | 12.1 MB | same |
| Trindle Toolkit, program built with fat LTO | 16 ms | 12.9 MB | 9.0 MB | `lto = "fat"`, `codegen-units = 1` in the program's release profile |
| Iced 0.14 (software, tiny-skia, Wayland) | 12, 15, 14 ms | 10.6 MB | 6.6 MB | Rust, built here with Rust 1.95 |
| Qt 6.12 Widgets (PySide6, Wayland software) | 13, 13, 14 ms | about 17 MB over a Python base of 41 MB | not compared (Python wheel) | Python |
| GTK 4.18 (Python, Mesa GL) | 575, 595, 924 ms | 145 MB over a Python base of 45 MB | not compared | Python |

Where Trindle is worse: first window is 2 to 5 ms behind Qt and Iced; resident memory is 2 to 4 MB above Iced; the binary is almost twice Iced's (screen reader link, bus, language tables, rich text shaping). Not measured: libcosmic (no copy here; it is built on Iced, so its figures would sit at or above Iced's), whole-page paint and idle CPU against any rival (each would need the same 1280 x 800 page built in it).

The fix: the toolkit now asks fontconfig about its usual family at the start of the font search, not at the first drawing. Before, the first drawing waited on the `fc-match` program, about 14 ms. The 8 ms font search and 3 ms first layout are what remain before the first drawing (2 ms).

## Closing the gaps with Qt and Iced (28 Oct, later)
- First buffer: the font search (8 ms) now starts on its own thread as the program connects to the compositor (`text::warm_fonts`, called from `Display::connect`), and fontconfig is asked early (earlier fix). 15 runs each, same minute: toolkit median 14.3 to 15.5 ms, Iced 15.3 ms, Qt 13 ms (earlier run). Level with Iced, about 2 ms behind Qt; the rest is the 3 ms first lay out and 2 ms first drawing, which are work the window needs.
- Memory and binary, all four built with the same flags (`lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `opt-level = "s"`), stripped:

| | Binary | Resident |
|---|---|---|
| Trindle Toolkit, default flags | 12.1 MB | 15.2 MB |
| Trindle Toolkit, small flags | 6.9 MB | 11.3 MB |
| Iced 0.14, default flags | 6.6 MB | 10.6 MB |
| Iced 0.14, small flags | 4.1 MB | 8.7 MB |

Why Trindle stays about 2.8 MB bigger in the binary and 2.6 MB in memory, by `cargo bloat` on the default build: the screen reader link (accesskit_unix, zbus and atspi, about 2.0 MB of code) is a requirement (WCAG 2.2, every window readable) that Iced's hello does not have; the picture formats (png, jpeg, webp, tiff, gif, bmp: about 0.7 MB) are used by Scan, Snippets and others; complex-script shaping and right-to-left text (rustybuzz, skrifa, bidi: about 0.6 MB) are required for the 15 languages. A program that wants none of those can turn the picture formats off in a later release; this is a trade for features the project needs, not waste.

## Requests from main (REQUESTS-FROM-MAIN.md), 28 Oct
1. Opaque region: after every whole painting the toolkit checks that every pixel is opaque (0.2 ms) and then sets `wl_surface.set_opaque_region` to the whole window; it clears it again if a window becomes see-through. Seen on the wire with `WAYLAND_DEBUG` on `hello`. Buffers stay Argb8888 so a see-through window works without a new buffer.
2. Damage: a whole painting now sends only the boxes that differ from the frame the compositor holds (bands of 16 rows, joined, at most 12 boxes, else one box round them); an unchanged frame sends one pixel. Patches already sent only their parts. Test: `only_the_boxes_that_changed_are_handed_over_as_damaged`.
3. Resizing: one painting per frame callback and the newest size only was already so (a size change only marks the window dirty; the painting uses the latest size).
4. Buffers: one pool of two buffers per window, made again only when the size changes; nothing is allocated per frame. No change needed.

## Crisper text on a 100% screen (29 Oct)

A tester saw the toolkit's words as thick and soft beside a browser's. A line of Trindle Sans at 14 px was drawn by plain FreeType with light hinting (what the desktop's fontconfig gives a browser) and by the toolkit, and the pixels were counted: solid pixels (above 90% cover) and half-grey ones (15 to 85%).

| | solid | half-grey |
|---|---|---|
| FreeType, light hinting | 362 | 786 |
| Toolkit, quarter-pixel placement | 316 | 861 |
| Toolkit, whole-pixel placement | 344 | 799 |

The ink in total was the same in all three. Letters now sit on whole pixels along the line as well as up and down; firmer edges and thinner strokes were tried and left out. The tester compared real windows beside a browser window on the 1920 by 1080 desktop at 100% and chose whole-pixel placement.

## Final figures, 30 Oct 2026 (quiet machine)
Machine: Framework 13, AMD Radeon 760M (radeonsi), Mesa 25.0.7, 12 cores, Rust 1.85, release build, nothing else running (load average under 1 at the start, 2 to 3 after), no `nice`. Each test was run three times; the figure is the median of the three runs' medians. The first run of the three is the least settled (its 1280 x 800 whole paint read 3.7 ms against 2.6 and 2.5), so the spread is about a fifth at the small sizes and a tenth at 4K. Battery use is not measured here: ten minutes of scrolling on battery with each drawer is the tester's test.

What the tests play: `measure_big_screens` builds the page of cards (heading, words, three buttons, switch, slider, entry) from real widgets at 200%, which `Ui::paint` records onto a `Painter` and the processor drawer (tiny-skia) plays. `measure_gpu_and_processor` plays a list written out by hand to look like the same page (not the real widgets recorded), once with the processor drawer and once with the GPU drawer (OpenGL ES 3.0, surfaceless EGL), and checks the two pictures are within 2 levels of each other. "GPU, processor time" is the thread's own time handing the frame over; "GPU, until drawn" is wall time to `glFinish`, with the processor idle. The test for the GPU drawing the real widgets does not exist yet, so the two tables below are not one comparison.

Commands, run from the repository: `cargo +1.85 test --release --lib measure_big_screens -- --ignored --nocapture`, `... measure_gpu_and_processor ...`, `... measure_card_shadows ...`, `cargo +1.85 test --release --lib --features ui,testing measure_long_lists -- --ignored --nocapture` (src/list_cost_tests.rs) and `... widgets_tests::measure_long_lists ...` (src/ui/widgets_tests.rs).

Real widgets, processor drawer (`measure_big_screens`):

| Window | Whole paint | Clear, swap, opacity check | Scroll step and paint | Spinner part | One core at 60 a second |
|---|---|---|---|---|---|
| 1280 x 800 | 2.6 ms | 0.4 ms | 7.9 ms | 0.37 ms | 18 per cent |
| 1920 x 1080 | 6.0 ms | 0.8 ms | 18.0 ms | 0.42 ms | 41 per cent |
| 2256 x 1504 | 11.5 ms | 1.5 ms | 32.3 ms | 0.46 ms | 78 per cent |
| 3840 x 2160 | 37.4 ms | 5.2 ms | 81.8 ms | 0.64 ms | 256 per cent |

The same sizes, a hand-written list of the same page, each drawer (`measure_gpu_and_processor`):

| Window | Processor drawer plays the list | GPU drawer, processor time | GPU drawer, until drawn |
|---|---|---|---|
| 1280 x 800 | 1.4 ms | 0.07 ms | 1.08 ms |
| 1920 x 1080 | 3.1 ms | 0.15 ms | 2.00 ms |
| 2256 x 1504 | 6.9 ms | 0.26 ms | 3.08 ms |
| 3840 x 2160 | 20.6 ms | 0.46 ms | 5.08 ms |

What the two tables say: at 4K the processor needs 37 ms to record and play the real page, over two frames at 60 a second; the GPU drawer takes 0.5 ms of processor time and 5 ms in all for a list of the same size, so the processor is free for the program, and the whole frame fits one refresh. That is for a whole repaint. A scroll step on the processor still costs 82 ms at 4K (it repaints the page); the GPU test of a scroll step on real widgets is not written. A spinner or pointer move paints only the parts that changed on either drawer (0.4 to 0.6 ms on the processor).

Card shadows (`measure_card_shadows`, 24 empty cards at 1920 x 1080, processor): whole paint 3.43 ms, the 24 shadows alone 2.57 ms (three runs: 3.40 to 3.45 and 2.57 each).

Long lists, build, lay out and paint a 400 by 300 window once (`widgets_tests::measure_long_lists`, three runs):

| What | Result |
|---|---|
| `list` of 1,000 rows | 12.6 ms |
| `list` of 10,000 rows | 10.3 ms |
| `list` of 100,000 rows | 16.0 ms |
| Painted table of 10,000 rows | 9.7 ms |
| Painted table of 100,000 rows | 9.2 ms |
| Painted table of 1,000,000 rows | 9.5 ms |

Shared pictures and nested rows (`list_cost_tests::measure_long_lists`, three runs):

| Page | Build | Lay out | Lay out and paint |
|---|---|---|---|
| 50 avatars, initials | 0.01 ms | 0.09 ms | 10.1 ms |
| 50 avatars, 1024 x 768 photo | 0.01 ms | 0.09 ms | 7.7 ms |
| 500 avatars, initials | 0.15 ms | 1.09 ms | 63.5 ms |
| 500 avatars, 1024 x 768 photo | 0.17 ms | 1.41 ms | 44.2 ms |
| 200 rows 1 deep | | 0.67 ms | 7.3 ms |
| 200 rows 2 deep | | 1.10 ms | 8.9 ms |
| 200 rows 3 deep | | 1.28 ms | 10.3 ms |
| 200 rows 4 deep | | 1.96 ms | 12.2 ms |
| 200 rows 5 deep | | 2.92 ms | 14.7 ms |

These replace the earlier long-list, big-screen and GPU tables, which were taken while other sessions were building. A million rows in a `list` was not tried: a million `Item`s are hundreds of megabytes before anything is drawn. Use `table::Table` for long data.

## The drawing service, 2 Nov 2026

Machine: Framework 13, Radeon 760M, Mesa 25.0.7, Rust 1.85, release build, other work on it throughout (load average about 19 on 12 cores), so frame rates and times are not clean. Test: `cargo +1.85 build --release --bin trindle-draw --example bench`, `./target/release/examples/bench 20 --programs`: that many copies of the bench at once, each 1575 x 1225 pixels. Memory in MB. "Shell" is the compositor, which also holds whatever else is on the desktop.

| Drawn by | Programs | Frames a second each | PSS, programs and service | Of it the service | GPU memory, programs and service | Shell PSS | Shell GPU memory | Processor time a frame, ms, programs + service |
|---|---|---|---|---|---|---|---|---|
| Processor | 1 | 57.7 | 33.4 | | 0 | 59.0 | 374.7 | 14.08 |
| Processor | 3 | 29.7 | 75.7 | | 0 | 66.5 | 435.2 | 20.48 |
| Processor | 6 | 13.1 | 139.8 | | 0 | 79.9 | 426.3 | 24.76 |
| GPU in the program | 1 | 50.4 | 63.0 | | 78.1 | 57.2 | 385.6 | 6.73 |
| GPU in the program | 3 | 41.0 | 162.3 | | 222.4 | 54.5 | 422.3 | 6.78 |
| GPU in the program | 6 | 32.5 | 310.5 | | 434.9 | 54.4 | 471.3 | 7.60 |
| Service | 1 | 54.4 | 66.9 | 40.0 | 79.2 | 57.2 | 383.6 | 4.55 + 2.55 |
| Service | 3 | 51.0 | 95.9 | 41.8 | 125.9 | 54.9 | 420.3 | 5.22 + 2.55 |
| Service | 6 | 33.4 | 138.1 | 40.7 | 188.0 | 55.0 | 473.3 | 5.69 + 2.54 |

What the numbers say:
- Against the GPU in each program, six programs save 172 MB of PSS and 247 MB of GPU memory, for about the same processor time (8.2 ms a frame against 7.6; at one program 7.1 against 6.7). With one program there is no saving: the service costs what the program would have.
- Against the processor, six programs hold the same PSS (138 against 140) with a third of the processor time a frame. Counting GPU memory they do not hold the same: the processor's programs have no GPU memory of their own, while the service holds 188 MB, of which about 92 MB is the windows' buffers and about 60 MB the driver's own. Adding PSS and GPU memory of programs, service and shell, and the buffers once though both sides count them, six programs come to about 760 MB on the service, 650 MB on the processor and 1180 MB on the GPU in each program. The shell's own figure moves by tens of MB between runs.
- A program adds about 16 MB of PSS on the service against about 50 MB on the GPU in the program, and about 22 MB of GPU memory against about 72 MB.
- Frames a second fall with six windows on this busy machine whichever draws, least on the GPU. The processor's frames take most of the processor's time, so it falls furthest.
- Not counted: the compositor's processor time, and the GPU's own time.

Hostile frames (`hostile_frames_are_drawn_within_a_limit_of_memory_and_hold_no_one_up`) are drawn by a service held to 900 MB of private memory: it never held above about 125 MB, against 41 MB idle. The other program's frame, drawn and read back, took at most 49 ms with the load average at 11 and 706 ms at 20, run at low priority, so most of that is the rest of the machine. A single upload (up to 40 MB in one frame) is not cut into slices.

The same test again after the fixes from an outside review, on a calmer machine (load average about 8, the compositor freshly started):

| Drawn by | Programs | Frames a second each | PSS, programs and service | Of it the service | GPU memory, programs and service | Shell PSS | Shell GPU memory | Processor time a frame, ms, programs + service |
|---|---|---|---|---|---|---|---|---|
| Processor | 1 | 60.0 | 33.2 | | 0 | 108.2 | 108.4 | 9.86 |
| Processor | 3 | 58.7 | 80.0 | | 0 | 116.2 | 127.4 | 11.38 |
| Processor | 6 | 25.8 | 139.8 | | 0 | 130.0 | 160.0 | 18.80 |
| GPU in the program | 1 | 60.0 | 59.7 | | 74.1 | 104.5 | 117.3 | 3.43 |
| GPU in the program | 3 | 60.0 | 156.8 | | 220.4 | 103.8 | 156.0 | 4.81 |
| GPU in the program | 6 | 50.6 | 306.9 | | 428.9 | 105.9 | 205.1 | 5.93 |
| Service | 1 | 59.9 | 63.9 | 36.9 | 81.2 | 104.7 | 119.3 | 3.66 + 1.47 |
| Service | 3 | 55.9 | 92.5 | 37.9 | 125.9 | 104.3 | 154.0 | 3.71 + 1.38 |
| Service | 6 | 50.2 | 138.4 | 38.0 | 192.0 | 104.4 | 207.1 | 4.00 + 1.20 |

At six programs the service holds level with the processor on PSS and takes under a third of its processor time at twice the frames. Counting GPU memory it comes to about 120 MB more than the processor (programs, service and shell, the windows' buffers once) and about 400 MB less than the GPU in each program. The shell's own PSS was about 45 MB in the run above and about 105 MB here, so a total for the whole desktop cannot be read from these runs, and the bench's windows are simpler than real programs.

## The 30 October reviews' performance claims, measured on 3 Nov 2026

Test file: `src/perf_checks_tests.rs`, run with `cargo +1.85 test --release --lib --features ui,testing perf_checks_tests -- --ignored --nocapture --test-threads=1` (and `list_cost_tests::measure_long_lists`). Processor time of the calling thread, the median of 5 to 9 runs. The machine was shared with other builds (load average 6 to 20), so every figure is rough; the before and after figures below were taken by running the old and the new test program by turns within minutes of each other, and are the range over three rounds.

Fixed:

| Claim | Before | After |
|---|---|---|
| Avatars and other drawings of the toolkit are painted although scrolled out of sight (500 rows, a window showing seven, 1280 x 800 at 2x) | avatar with initials 65 to 128 ms, avatar with photo 38 to 66 ms, `image_view` 17 to 19 ms, `icon_circle` 14 to 16 ms, `icon_image` 10 to 12 ms | 7 to 13 ms, 7 to 9 ms, 6 to 8 ms, 6 ms, 6 ms (the rest is the 500 rows of words) |
| `Sketch::shadow` of a 200 x 120 shape | 1.6 ms on 400 x 300, 27 to 31 ms on 1920 x 1080, 142 to 148 ms on 3840 x 2160 | 0.7 ms, 0.7 to 0.8 ms, 0.8 to 1.1 ms |
| `Sketch::clipped` of the same | 0.4 ms, 5.7 to 6.6 ms, 24 to 26 ms | 0.2 ms, 0.3 to 0.6 ms, 2.7 to 3.0 ms (the empty layer is still made) |
| `ListModel` sorted and grouped, 100,000 things | 118 to 148 ms | 33 to 36 ms (sorted alone is 16 to 25 ms, unchanged) |

Hangs, fixed without a number: `open_with::recent` looked at every recent file with `path.exists()`, which waits as long as a share stays silent; `bus::Asked::reply`, `fail` and drop waited up to 5 seconds on the answering thread when the bus daemon had stopped taking messages; every sheet file event started a thread that read the sheets, so a burst of saves ran many at once and an older reading could land over a newer one. Measured only for the third: one reading of the sheets takes 0.3 to 0.5 ms.

Claim does not stand, or does not matter, at the sizes measured:

| Claim | Number |
|---|---|
| Avatar crops the picture on every description | The crop and scaling are kept per picture (`picture::shared`): 500 avatars build in 0.2 ms and lay out in 1.1 to 1.5 ms. The real cost was painting those off screen, fixed above |
| Nested rows measure each item several times, threefold a level | 200 rows 1 to 5 deep lay out in 0.7, 1.4, 1.4, 2.4 and 3.0 ms (rounds on a quiet machine): about 0.6 ms a level, not threefold. No cache added |
| `area.rs` scans fifty lines or more with the spell checker on every view build | The scan is of the lines in sight and 50 more, so 1,000 and 100,000 lines both cost 0.01 ms. One line of 2 MB costs 5 ms a view build, in a minified file only |
| Icon grid select clones the selection and tests every tile | 10,000 tiles: 0.09 ms a band move, 0.4 ms with all chosen first. 100,000 tiles: 0.9 ms a band move, 5 to 8 ms with all 100,000 chosen first, which is in a frame but not by much. Not changed. Recording a paint of 100,000 tiles is 3.3 ms, as for 1,000 |
| `sharp.rs` polls fc-match with a 2 ms sleep on the first draw | It runs on its own thread since 7737840, not the window's |
| Font re-read | Fonts are read once and their covers kept. The first look at a letter that no chosen font has reads the covers of the fonts in turn: 14 to 22 ms once for the first Chinese letter on this machine (405 faces, none with Chinese), 0.05 ms for the next, on a warm disk. A cold disk was not measured. `chooser()` 3.6 to 5.2 ms and `lang::script::system_families()` 8 to 12 ms (documented as off the main thread) |
| Mask store emptied whole at 4,096 letters | A letter not yet drawn costs 6 to 11 us, so refilling 1,500 letters in sight is 9 to 17 ms once, after more than 4,096 new letter, size and weight combinations. Not changed |
| `first_weekday` parsed for each cell | Read once at the top of `month_grid` already; one call is 0.04 to 0.09 us |
| Text cache key copies the string | A lookup that hits costs 0.1 to 0.2 us for 20 letters, 0.3 to 0.6 us for 200, 8 to 18 us for 5,000 |
| `Stays` is quadratic | 10, 100 and 1,000 floating places laid out: 0.007, 0.07 and 1.9 to 3.1 ms; a program has a handful |
