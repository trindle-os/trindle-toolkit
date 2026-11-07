//! A repeatable drawing benchmark for the processor, the GPU drawer and the drawing service (`TRINDLE_DRAWN_BY_GPU=1 cargo run --release --example bench -- 20`, or `--both`, or `--programs` for 1, 3 and 6 programs of each); needs a Wayland desktop and windows that are on screen.

use std::cell::RefCell;
use std::collections::HashMap;
use std::process::{Command, ExitCode, Stdio};
use std::rc::Rc;
use std::time::{Duration, Instant};

use trindle_toolkit::bars::progress_bar;
use trindle_toolkit::event::Loop;
use trindle_toolkit::t;
use trindle_toolkit::ui::{App, Drawer, Painted, Shown};
use trindle_toolkit::widget::{column, entry, item, label, list, row, scroll, spinner, Item, Node};
use trindle_toolkit::window::{Display, Kind};

const WARM_UP: f32 = 1.0;
const ROWS: usize = 10_000;
const SPEED: f32 = 1200.0;
const RANGE: f32 = 6000.0;
const FRAME_MS: f64 = 1000.0 / 60.0;
const SENTENCE: &str = "The quick brown fox jumps over the lazy dog";
const NAMES: [&str; 8] = ["John Smith", "Jane Brown", "Alan Taylor", "Susan Wilson", "David Evans", "Mary Thomas", "Peter Roberts", "Helen Walker"];
const ICONS: [&str; 4] = ["folder-symbolic", "text-x-generic-symbolic", "document-open-symbolic", "user-trash-symbolic"];

const METRICS: [(&str, &str, &str, usize); 21] = [
    ("frames", "Frames drawn", "", 0),
    ("fps", "Frames a second", "", 1),
    ("avg", "Frame time, average", "ms", 2),
    ("p95", "Frame time, 95th percentile", "ms", 2),
    ("worst", "Frame time, worst", "ms", 2),
    ("over", "Frames over 16.7 ms", "", 0),
    ("gap_avg", "Between frames, average", "ms", 2),
    ("gap_p95", "Between frames, 95th percentile", "ms", 2),
    ("gap_worst", "Between frames, worst", "ms", 2),
    ("update", "Update and layout, average (same for both)", "ms", 2),
    ("user", "CPU time, user", "s", 2),
    ("system", "CPU time, system", "s", 2),
    ("cpu", "CPU time as a share of one core", "%", 1),
    ("rss_peak", "Memory RSS, peak", "MB", 1),
    ("rss_final", "Memory RSS, final", "MB", 1),
    ("pss_peak", "Memory PSS, peak", "MB", 1),
    ("pss_final", "Memory PSS, final", "MB", 1),
    ("by_gpu", "Frames the GPU drew", "", 0),
    ("by_cpu", "Frames the processor drew", "", 0),
    ("by_service", "Frames the drawing service drew", "", 0),
    ("seconds", "Seconds measured", "s", 1),
];

struct Bench {
    items: Vec<Item>,
    seconds: f32,
}

#[derive(Clone)]
enum Msg {
    Nothing,
}

impl App for Bench {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Nothing => {}
        }
    }

    fn view(&self) -> Node<Msg> {
        let t = self.seconds;
        let typed: String = SENTENCE.chars().take((t * 15.0) as usize % (SENTENCE.len() + 10)).collect();
        let clock = t!("kit-example-bench-clock", secs = format!("{t:.1}"));
        column(vec![
            row(vec![spinner(), label(clock)]).gap(12.0),
            progress_bar(Some(t % 4.0 / 4.0), t!("kit-example-bench-progress")),
            entry(typed, t!("kit-example-bench-hint"), |_| Msg::Nothing),
            scroll("rows", list(self.items.clone(), None, |_| Msg::Nothing).called(t!("kit-example-bench-rows"))),
        ])
        .padding(16.0)
        .gap(12.0)
    }
}

fn items() -> Vec<Item> {
    (0..ROWS)
        .map(|i| {
            let name = NAMES[i % NAMES.len()];
            let mail = name.to_lowercase().replace(' ', ".");
            item(format!("{name} {i}")).icon(ICONS[i % ICONS.len()]).detail(format!("{mail}.{i}@example.com"))
        })
        .collect()
}

fn scroll_at(t: f32) -> f32 {
    let along = (t * SPEED) % (2.0 * RANGE);
    if along < RANGE { along } else { 2.0 * RANGE - along }
}

#[derive(Default)]
struct Stats {
    first: Option<Instant>,
    began: Option<Instant>,
    last: Option<Instant>,
    took: Vec<f64>,
    gaps: Vec<f64>,
    updates: Vec<f64>,
    by_gpu: u32,
    by_cpu: u32,
    by_service: u32,
    size: (u32, u32),
    scale: f32,
    cpu_before: (f64, f64),
    pss_peak: f64,
    ended: Option<(f64, f64, f64)>,
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn cpu_now() -> (f64, f64) {
    let text = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let after = text.rsplit_once(')').map_or("", |(_, rest)| rest);
    let field = |n: usize| after.split_whitespace().nth(n).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0) / 100.0;
    (field(11), field(12))
}

fn proc_mb(file: &str, key: &str) -> f64 {
    let text = std::fs::read_to_string(file).unwrap_or_default();
    let kb = text.lines().find_map(|l| l.strip_prefix(key)?.trim_start_matches(':').split_whitespace().next()?.parse::<f64>().ok());
    kb.unwrap_or(0.0) / 1024.0
}

fn percentile(sorted: &[f64], share: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[((sorted.len() as f64 * share).ceil() as usize).clamp(1, sorted.len()) - 1]
}

fn sorted(v: &[f64]) -> Vec<f64> {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    v
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 }
}

struct Report {
    drawer: String,
    size: String,
    scale: f32,
    numbers: Vec<f64>,
}

fn report(s: &Stats, how: &Drawer) -> Report {
    let (took, gaps) = (sorted(&s.took), sorted(&s.gaps));
    let (user, system, wall) = s.ended.map(|(u, y, w)| (u - s.cpu_before.0, y - s.cpu_before.1, w)).unwrap_or_default();
    let frames = s.took.len() as f64;
    let numbers = vec![
        frames,
        if wall > 0.0 { frames / wall } else { 0.0 },
        mean(&took),
        percentile(&took, 0.95),
        took.last().copied().unwrap_or(0.0),
        took.iter().filter(|&&t| t > FRAME_MS).count() as f64,
        mean(&gaps),
        percentile(&gaps, 0.95),
        gaps.last().copied().unwrap_or(0.0),
        mean(&s.updates),
        user,
        system,
        if wall > 0.0 { (user + system) / wall * 100.0 } else { 0.0 },
        proc_mb("/proc/self/status", "VmHWM"),
        proc_mb("/proc/self/smaps_rollup", "Rss"),
        s.pss_peak.max(proc_mb("/proc/self/smaps_rollup", "Pss")),
        proc_mb("/proc/self/smaps_rollup", "Pss"),
        s.by_gpu as f64,
        s.by_cpu as f64,
        s.by_service as f64,
        wall,
    ];
    let drawer = match (how, s.by_gpu, s.by_cpu) {
        (Drawer::Service, g, _) if s.by_service == g && g > 0 => "the drawing service (every GPU frame)".to_string(),
        (Drawer::Service, g, c) => format!("the drawing service for {} frames, then the GPU here for {} and the processor for {c}", s.by_service, g - s.by_service),
        (Drawer::Gpu, g, 0) if g > 0 => "GPU (every frame)".to_string(),
        (Drawer::Gpu, g, c) if g > 0 => format!("GPU, then the processor took over after {g} frames ({c} frames by the processor): the GPU failed"),
        (Drawer::Gpu, _, _) => "processor (the GPU was chosen but drew no frame)".to_string(),
        (Drawer::Processor(why), _, _) => format!("processor (fallback reason: {})", why.words()),
    };
    Report { drawer, size: format!("{} x {} pixels", s.size.0, s.size.1), scale: s.scale, numbers }
}

fn show(v: f64, decimals: usize, unit: &str) -> String {
    if unit.is_empty() { format!("{v:.decimals$}") } else { format!("{v:.decimals$} {unit}") }
}

fn print_one(r: &Report, seconds: f32) {
    println!("Trindle drawing benchmark");
    println!("Drawer asked for:  {}", asked());
    println!("Drawer used:       {}", r.drawer);
    println!("Resolution:        {}, scale {:.2}", r.size, r.scale);
    println!("Run:               {seconds:.0} s measured, starting {WARM_UP:.0} s after the first frame (earlier frames not counted)");
    for (n, (_, name, unit, d)) in r.numbers.iter().zip(METRICS) {
        println!("{:<44} {}", format!("{name}:"), show(*n, d, unit));
    }
}

fn asked() -> &'static str {
    let on = |k: &str| std::env::var_os(k).is_some_and(|v| v == "1");
    if on("TRINDLE_DRAWN_BY_PROCESSOR") {
        "processor (TRINDLE_DRAWN_BY_PROCESSOR=1)"
    } else if on("TRINDLE_DRAWN_BY_SERVICE") {
        "the drawing service (TRINDLE_DRAWN_BY_SERVICE=1)"
    } else if on("TRINDLE_DRAWN_BY_GPU") {
        "GPU (TRINDLE_DRAWN_BY_GPU=1)"
    } else {
        "processor (the default)"
    }
}

fn print_child(r: &Report) {
    println!("drawer={}\nsize={}\nscale={}", r.drawer, r.size, r.scale);
    for (n, (key, ..)) in r.numbers.iter().zip(METRICS) {
        println!("{key}={n}");
    }
}

fn run(seconds: f32) -> Option<Report> {
    let events = Loop::new();
    let gone = events.clone();
    let Some(display) = Display::connect(&events, move || gone.quit()) else {
        eprintln!("{}", t!("kit-run-no-desktop"));
        return None;
    };
    let kind = Kind::Window { title: t!("kit-example-bench-title").into(), app_id: "org.example.Bench".into() };
    let shown = Shown::open(&events, &display, kind, (900, 700), Bench { items: items(), seconds: 0.0 });
    let Some(shown) = shown else {
        eprintln!("{}", t!("kit-run-no-window"));
        return None;
    };
    let shown = Rc::new(shown);
    let quit = events.clone();
    shown.window.on_close(move || quit.quit());
    let stats = Rc::new(RefCell::new(Stats::default()));

    let (hook_stats, weak, ev) = (stats.clone(), Rc::downgrade(&shown), events.clone());
    let finish = {
        let (done, quit) = (stats.clone(), events.clone());
        move || {
            let (user, system) = cpu_now();
            let mut s = done.borrow_mut();
            let wall = s.began.map_or(0.0, |b| b.elapsed().as_secs_f64());
            s.ended = Some((user, system, wall));
            quit.quit();
        }
    };
    let mut finish = Some(finish);
    shown.on_painted(move |p: &Painted| {
        let mut s = hook_stats.borrow_mut();
        let first = *s.first.get_or_insert(p.began);
        if p.began.saturating_duration_since(first) >= Duration::from_secs_f32(WARM_UP) && s.began.is_none() {
            s.began = Some(p.began);
            s.cpu_before = cpu_now();
            if let Some(finish) = finish.take() {
                ev.after(Duration::from_secs_f32(seconds), finish);
            }
        }
        if s.began.is_some() && s.ended.is_none() {
            s.took.push(ms(p.took));
            if let Some(last) = s.last {
                s.gaps.push(ms(p.began.saturating_duration_since(last)));
            }
            s.last = Some(p.began);
            if p.by_gpu { s.by_gpu += 1 } else { s.by_cpu += 1 }
            s.by_service += u32::from(p.by_service);
            (s.size, s.scale) = (p.size, p.scale);
        }
        let scale = p.scale;
        if s.ended.is_none() {
            drop(s);
            let (weak, stats) = (weak.clone(), hook_stats.clone());
            ev.after(Duration::ZERO, move || {
                let Some(shown) = weak.upgrade() else { return };
                let (t, before) = (first.elapsed().as_secs_f32(), Instant::now());
                shown.change(|b| b.seconds = t);
                shown.scroll_area("rows", scroll_at(t) * scale);
                let mut s = stats.borrow_mut();
                if s.began.is_some() {
                    s.updates.push(ms(before.elapsed()));
                }
            });
        }
    });

    let (waiting, quit) = (stats.clone(), events.clone());
    events.after(Duration::from_secs(30), move || {
        if waiting.borrow().began.is_none() {
            quit.quit();
        }
    });
    let sampled = stats.clone();
    events.every(Duration::from_millis(500), move || {
        let mut s = sampled.borrow_mut();
        s.pss_peak = s.pss_peak.max(proc_mb("/proc/self/smaps_rollup", "Pss"));
        trindle_toolkit::event::Flow::Continue
    });
    shown.change(|b| b.seconds = 0.0);
    events.run();

    let s = stats.borrow();
    if s.took.is_empty() {
        eprintln!("No frame was drawn in the measured time. Is the window on screen, not hidden or on another workspace?");
        return None;
    }
    Some(report(&s, shown.drawer()))
}

fn child(seconds: f32, gpu: bool) -> Result<(HashMap<String, String>, Vec<f64>), String> {
    let mut cmd = Command::new(std::env::current_exe().map_err(|e| e.to_string())?);
    cmd.arg(seconds.to_string()).arg("--child").env_remove("TRINDLE_DRAWN_BY_PROCESSOR").env_remove("TRINDLE_DRAWN_BY_GPU");
    if gpu {
        cmd.env("TRINDLE_DRAWN_BY_GPU", "1");
    }
    let out = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output().map_err(|e| e.to_string())?;
    let tail = |o: &std::process::Output| {
        let err = String::from_utf8_lossy(&o.stderr);
        let lines: Vec<&str> = err.lines().collect();
        lines[lines.len().saturating_sub(12)..].join("\n")
    };
    if !out.status.success() {
        return Err(format!("it ended with {}; its last messages:\n{}", out.status, tail(&out)));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let words: HashMap<String, String> = text.lines().filter_map(|l| l.split_once('=')).map(|(k, v)| (k.to_string(), v.to_string())).collect();
    let numbers = METRICS.iter().map(|(k, ..)| words.get(*k).and_then(|v| v.parse().ok())).collect::<Option<Vec<f64>>>();
    match numbers {
        Some(n) => Ok((words, n)),
        None => Err(format!("it printed no result; its last messages:\n{}", tail(&out))),
    }
}

fn both(seconds: f32) -> ExitCode {
    eprintln!("Running with the processor, then with the GPU: about {:.0} s in all.", 2.0 * (seconds + WARM_UP) + 4.0);
    let mut runs = Vec::new();
    for (name, gpu) in [("processor", false), ("GPU", true)] {
        match child(seconds, gpu) {
            Ok(r) => runs.push(r),
            Err(why) => {
                eprintln!("The {name} run failed (the benchmark needs a Wayland desktop with the window on screen): {why}");
                return ExitCode::FAILURE;
            }
        }
    }
    let (gpu, cpu) = (runs.pop().unwrap(), runs.pop().unwrap());
    println!("Trindle drawing benchmark: processor and GPU side by side");
    println!("{seconds:.0} s measured, starting {WARM_UP:.0} s after the first frame; the processor run is the default drawer, the GPU run has TRINDLE_DRAWN_BY_GPU=1");
    println!("Drawer used, processor run: {}", cpu.0["drawer"]);
    println!("Drawer used, GPU run:       {}", gpu.0["drawer"]);
    println!("Resolution, processor run:  {}, scale {}", cpu.0["size"], cpu.0["scale"]);
    println!("Resolution, GPU run:        {}, scale {}", gpu.0["size"], gpu.0["scale"]);
    println!("{:<44} {:>14} {:>14} {:>16}", "", "Processor", "GPU", "GPU minus proc.");
    for ((a, b), (_, name, unit, d)) in cpu.1.iter().zip(&gpu.1).zip(METRICS) {
        let change = if *a != 0.0 { format!(" ({:+.0}%)", (b - a) / a * 100.0) } else { String::new() };
        println!("{:<44} {:>14} {:>14} {:>16}", name, show(*a, d, unit), show(*b, d, unit), format!("{}{change}", show(b - a, d, unit)));
    }
    ExitCode::SUCCESS
}

#[derive(Default)]
struct Set {
    pss_clients: f64,
    pss_service: f64,
    pss_compositor: f64,
    gpu_memory: f64,
    gpu_compositor: f64,
    frames: f64,
    fps: f64,
    client_ms: f64,
    service_ms: f64,
    by: [f64; 3],
}

fn cpu_of(pid: u32) -> f64 {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
    let after = text.rsplit_once(')').map_or("", |(_, rest)| rest);
    let field = |n: usize| after.split_whitespace().nth(n).and_then(|f| f.parse::<f64>().ok()).unwrap_or(0.0) / 100.0;
    field(11) + field(12)
}

fn pss_of(pid: u32) -> f64 {
    proc_mb(&format!("/proc/{pid}/smaps_rollup"), "Pss")
}

fn gpu_memory_of(pid: u32) -> f64 {
    let mut seen = HashMap::new();
    let Ok(fds) = std::fs::read_dir(format!("/proc/{pid}/fdinfo")) else { return 0.0 };
    for fd in fds.flatten() {
        let Ok(text) = std::fs::read_to_string(fd.path()) else { continue };
        let Some(id) = text.lines().find_map(|l| l.strip_prefix("drm-client-id:")).map(|v| v.trim().to_string()) else { continue };
        let kb: f64 = text.lines().filter(|l| l.starts_with("drm-memory-") || l.starts_with("drm-total-")).filter(|l| !l.starts_with("drm-memory-cpu") && !l.starts_with("drm-total-cpu"))
            .filter(|l| !text.contains("drm-total-") || l.starts_with("drm-total-"))
            .filter_map(|l| l.split_whitespace().nth(1)?.parse::<f64>().ok()).sum();
        seen.insert(id, kb);
    }
    seen.values().sum::<f64>() / 1024.0
}

fn compositor() -> Option<u32> {
    std::fs::read_dir("/proc").ok()?.flatten().find_map(|e| {
        let pid: u32 = e.file_name().to_str()?.parse().ok()?;
        (std::fs::read_to_string(e.path().join("comm")).ok()?.trim() == "trindle-compose").then_some(pid)
    })
}

fn run_set(seconds: f32, count: usize, env: Option<&str>, service: bool) -> Result<Set, String> {
    let mut server = None;
    if service {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let path = exe.parent().and_then(|p| p.parent()).ok_or("no folder")?.join("trindle-draw");
        let socket = std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").ok_or("no runtime folder")?).join("trindle-draw.socket");
        let _ = std::fs::remove_file(&socket);
        server = Some(Command::new(path).stdout(Stdio::null()).spawn().map_err(|e| format!("trindle-draw: {e}"))?);
        let began = Instant::now();
        while !socket.exists() {
            if began.elapsed() > Duration::from_secs(10) {
                return Err("trindle-draw made no socket".into());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut children = Vec::new();
    for _ in 0..count {
        let mut cmd = Command::new(&exe);
        cmd.arg(seconds.to_string()).arg("--child").env_remove("TRINDLE_DRAWN_BY_PROCESSOR").env_remove("TRINDLE_DRAWN_BY_GPU").env_remove("TRINDLE_DRAWN_BY_SERVICE");
        if let Some(key) = env {
            cmd.env(key, "1");
        }
        children.push(cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| e.to_string())?);
    }
    let pids: Vec<u32> = children.iter().map(|c| c.id()).collect();
    let server_pid = server.as_ref().map(|s| s.id());
    let shell = compositor();
    let started = Instant::now();
    let settle_for = Duration::from_secs_f32(WARM_UP + 3.0);
    let last = Duration::from_secs_f32(seconds - 1.0 + WARM_UP);
    std::thread::sleep(settle_for);
    let cpu_from = (Instant::now(), server_pid.map_or(0.0, cpu_of));
    let mut set = Set::default();
    while started.elapsed() < last {
        std::thread::sleep(Duration::from_millis(500));
        set.pss_clients = pids.iter().map(|p| pss_of(*p)).sum();
        set.pss_service = server_pid.map_or(0.0, pss_of);
        set.pss_compositor = shell.map_or(0.0, pss_of);
        set.gpu_compositor = shell.map_or(0.0, gpu_memory_of);
        set.gpu_memory = pids.iter().chain(&server_pid).map(|p| gpu_memory_of(*p)).sum();
    }
    let span = cpu_from.0.elapsed().as_secs_f64();
    let server_cpu = server_pid.map_or(0.0, cpu_of) - cpu_from.1;
    let (mut frames, mut client_cpu, mut wall) = (0.0, 0.0, 0.0f64);
    for child in children {
        let out = child.wait_with_output().map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&out.stdout);
        let got: HashMap<&str, f64> = text.lines().filter_map(|l| l.split_once('=')).filter_map(|(k, v)| Some((k, v.parse().ok()?))).collect();
        let number = |k: &str| got.get(k).copied().unwrap_or(0.0);
        if number("frames") == 0.0 {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!("a program drew nothing: {}", err.lines().rev().take(6).collect::<Vec<_>>().join(" / ")));
        }
        frames += number("frames");
        client_cpu += number("user") + number("system");
        wall = wall.max(number("seconds"));
        set.fps += number("fps");
        set.by[0] += number("by_service");
        set.by[1] += number("by_gpu") - number("by_service");
        set.by[2] += number("by_cpu");
    }
    if let Some(mut s) = server {
        let ended = s.try_wait().map_err(|e| e.to_string())?;
        if let Some(code) = ended {
            return Err(format!("the service ended early: {code}"));
        }
        let _ = s.kill();
        let _ = s.wait();
    }
    set.frames = frames;
    set.client_ms = if frames > 0.0 { client_cpu / frames * 1000.0 } else { 0.0 };
    set.service_ms = if set.fps > 0.0 { server_cpu / (set.fps * span) * 1000.0 } else { 0.0 };
    Ok(set)
}

fn programs(seconds: f32) -> ExitCode {
    println!("{seconds:.0} s each after a {WARM_UP:.0} s warm-up; PSS in MB, GPU memory in MB as the kernel counts it for these processes, processor time per frame in ms");
    println!("{:<10} {:>2} {:>8} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}  frames by service/GPU here/processor", "drawn by", "n", "fps each", "PSS total", "PSS each", "service", "PSS shell", "GPU mem", "GPU shell", "ms client", "ms svc", "ms total");
    for (name, env, service) in [("processor", None, false), ("GPU here", Some("TRINDLE_DRAWN_BY_GPU"), false), ("service", Some("TRINDLE_DRAWN_BY_SERVICE"), true)] {
        for count in [1usize, 3, 6] {
            match run_set(seconds, count, env, service) {
                Ok(s) => println!("{:<10} {:>2} {:>8.1} {:>9.1} {:>9.1} {:>9.1} {:>9.1} {:>9.1} {:>9.1} {:>9.2} {:>9.2} {:>9.2}  {:.0}/{:.0}/{:.0}", name, count, s.fps / count as f64, s.pss_clients + s.pss_service, s.pss_clients / count as f64, s.pss_service, s.pss_compositor, s.gpu_memory, s.gpu_compositor, s.client_ms, s.service_ms, s.client_ms + s.service_ms, s.by[0], s.by[1], s.by[2]),
                Err(why) => println!("{name:<10} {count:>2} failed: {why}"),
            }
        }
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seconds = args.iter().find_map(|a| a.parse::<f32>().ok()).filter(|s| *s > 0.0).unwrap_or(20.0);
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Usage: bench [SECONDS] [--both | --programs]\n  SECONDS     how long to measure, default 20\n  --both      run once with each drawer and print them side by side\n  --programs  1, 3 and 6 programs at once with each way of drawing, counting the drawing service\nThe drawer is chosen by the environment: TRINDLE_DRAWN_BY_SERVICE=1 for the drawing service, TRINDLE_DRAWN_BY_GPU=1 for the GPU, else the processor.");
        return ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--programs") {
        return programs(seconds);
    }
    if args.iter().any(|a| a == "--both") {
        return both(seconds);
    }
    let Some(r) = run(seconds) else { return ExitCode::FAILURE };
    if args.iter().any(|a| a == "--child") { print_child(&r) } else { print_one(&r, seconds) }
    ExitCode::SUCCESS
}
