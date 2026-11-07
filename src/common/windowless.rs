//! The Wayland setup for a program with no window: a connection, the registry and the seat, and waiting on the desktop for a time.

use std::os::fd::AsRawFd;
use std::time::Duration;

use wayland_client::backend::WaylandError;
use wayland_client::globals::{registry_queue_init, GlobalList, GlobalListContents};
use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, EventQueue};
use crate::t;

pub struct Windowless<S> {
    pub conn: Connection,
    pub globals: GlobalList,
    pub queue: EventQueue<S>,
    pub seat: wl_seat::WlSeat,
}

impl<S> Windowless<S>
where
    S: Dispatch<wl_registry::WlRegistry, GlobalListContents> + Dispatch<wl_seat::WlSeat, ()> + 'static,
{
    pub fn open() -> Result<Self, &'static str> {
        Self::on(Connection::connect_to_env().map_err(|_| t!("kit-windowless-no-desktop"))?)
    }

    pub fn on(conn: Connection) -> Result<Self, &'static str> {
        let (globals, queue) = registry_queue_init::<S>(&conn).map_err(|_| t!("kit-windowless-no-answer"))?;
        let seat = globals.bind(&queue.handle(), 1..=1, ()).map_err(|_| t!("kit-windowless-no-seat"))?;
        Ok(Windowless { conn, globals, queue, seat })
    }

    pub fn dispatch_for(&mut self, state: &mut S, wait: Option<Duration>) -> bool {
        let Some(wait) = wait else { return self.queue.blocking_dispatch(state).is_ok() };
        if self.queue.dispatch_pending(state).is_err() || self.conn.flush().is_err() {
            return false;
        }
        let Some(guard) = self.queue.prepare_read() else {
            return self.queue.dispatch_pending(state).is_ok();
        };
        let mut fd = libc::pollfd { fd: guard.connection_fd().as_raw_fd(), events: libc::POLLIN, revents: 0 };
        let ms = wait.as_millis().min(i32::MAX as u128) as i32;
        let ready = crate::sys::fd::poll(std::slice::from_mut(&mut fd), ms);
        if ready > 0 {
            match guard.read() {
                Err(WaylandError::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => return false,
                Ok(_) => {}
            }
        } else {
            drop(guard);
        }
        self.queue.dispatch_pending(state).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixStream;
    use std::sync::Arc;

    use wayland_client::{Connection, Proxy, QueueHandle};
    use wayland_server::backend::{ClientData, ClientId, DisconnectReason};
    use wayland_server::protocol::wl_seat as server_seat;
    use wayland_server::{Client, DataInit, DisplayHandle, GlobalDispatch, New};

    use super::*;

    struct Mine;

    impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Mine {
        fn event(_: &mut Self, _: &wl_registry::WlRegistry, _: wl_registry::Event, _: &GlobalListContents, _: &Connection, _: &QueueHandle<Self>) {}
    }

    impl Dispatch<wl_seat::WlSeat, ()> for Mine {
        fn event(_: &mut Self, _: &wl_seat::WlSeat, _: wl_seat::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
    }

    struct Stand;

    impl ClientData for Stand {
        fn initialized(&self, _: ClientId) {}
        fn disconnected(&self, _: ClientId, _: DisconnectReason) {}
    }

    impl GlobalDispatch<server_seat::WlSeat, ()> for Stand {
        fn bind(_: &mut Self, _: &DisplayHandle, _: &Client, new: New<server_seat::WlSeat>, _: &(), init: &mut DataInit<'_, Self>) {
            init.init(new, ());
        }
    }

    impl wayland_server::Dispatch<server_seat::WlSeat, ()> for Stand {
        fn request(_: &mut Self, _: &Client, _: &server_seat::WlSeat, _: server_seat::Request, _: &(), _: &DisplayHandle, _: &mut DataInit<'_, Self>) {}
    }

    fn desktop(with_seat: bool) -> Connection {
        let (ours, theirs) = UnixStream::pair().expect("a pair of sockets");
        let mut display = wayland_server::Display::<Stand>::new().expect("a stand-in desktop");
        if with_seat {
            display.handle().create_global::<Stand, server_seat::WlSeat, ()>(1, ());
        }
        display.handle().insert_client(theirs, Arc::new(Stand)).expect("the connection");
        std::thread::spawn(move || {
            let mut state = Stand;
            for _ in 0..400 {
                if display.dispatch_clients(&mut state).is_err() {
                    break;
                }
                let _ = display.flush_clients();
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        Connection::from_socket(ours).expect("a Wayland connection")
    }

    #[test]
    fn a_program_with_no_window_connects_and_finds_the_seat() {
        let mut setup = Windowless::<Mine>::on(desktop(true)).map_err(|e| e.to_string()).expect("set up");
        assert!(setup.seat.is_alive());
        let started = std::time::Instant::now();
        assert!(setup.dispatch_for(&mut Mine, Some(Duration::from_millis(20))), "a quiet desktop is not a gone one");
        assert!(started.elapsed() < Duration::from_secs(2), "the wait ran on");
    }

    #[test]
    fn a_desktop_with_no_seat_is_said_so() {
        assert_eq!(Windowless::<Mine>::on(desktop(false)).err(), Some("the desktop has no keyboard or mouse"));
    }
}
