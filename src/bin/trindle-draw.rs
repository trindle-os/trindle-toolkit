//! The drawing service: started by the session at login, it draws for every program that asks with `TRINDLE_DRAWN_BY_SERVICE=1`.

fn main() -> std::process::ExitCode {
    trindle_toolkit::draw_service()
}
