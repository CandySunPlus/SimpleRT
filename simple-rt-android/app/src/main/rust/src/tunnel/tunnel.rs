use super::binary;
use log::trace;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::prelude::{FromRawFd, RawFd};
use std::sync::{atomic, Arc, Mutex};
use std::thread;

#[allow(unused_macros)]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

#[derive(Debug)]
enum HandleType {
    Tun,
    Acc,
}

pub struct Tunnel {
    handles: Vec<Option<thread::JoinHandle<()>>>,
    is_started: Arc<atomic::AtomicBool>,
}

const ACC_BUF_SIZE: usize = 4096;

impl Tunnel {
    pub fn new() -> Self {
        Self {
            handles: vec![],
            is_started: Arc::new(atomic::AtomicBool::new(false)),
        }
    }

    fn get_file_with_raw_fd(fd: RawFd) -> Option<File> {
        if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFL, 0)) {
            syscall!(fcntl(fd, libc::F_SETFL, flags & !libc::O_NONBLOCK))
                .expect("set acc_fd flags failed");
            return Some(unsafe { File::from_raw_fd(fd) });
        }
        None
    }

    fn thread_proc(
        tun_file: &mut File,
        acc_file: &mut File,
        // tunnel: Arc<Mutex<Self>>,
        handle_type: HandleType,
    ) {
        let mut buf = [0u8; ACC_BUF_SIZE];
        let in_file: &mut File;
        let out_file: &mut File;
        match handle_type {
            HandleType::Tun => {
                trace!("start {:?} thread", handle_type);
                in_file = tun_file;
                out_file = acc_file;
            }
            HandleType::Acc => {
                trace!("start {:?} thread", handle_type);
                in_file = acc_file;
                out_file = tun_file;
            }
        }
        loop {
            if let Ok(_) = in_file.read(&mut buf) {
                out_file.write(&buf).expect("write file error");
                trace!(
                    "writing {} in {:?} mode",
                    binary::build_packet_string(&buf),
                    handle_type
                );
            } else {
                trace!("tunnel broken in {:?} mode", handle_type);
                break;
            }
        }
    }

    pub fn start(tunnel_rc: Arc<Mutex<Self>>, tun_fd: RawFd, acc_fd: RawFd) {
        let mut tunnel = tunnel_rc.lock().unwrap();
        tunnel.is_started.store(true, atomic::Ordering::SeqCst);

        for handle_type in vec![HandleType::Tun, HandleType::Acc] {
            let mut tun_file = Tunnel::get_file_with_raw_fd(tun_fd);
            let mut acc_file = Tunnel::get_file_with_raw_fd(acc_fd);
            tunnel.handles.push(Some(thread::spawn(move || {
                Tunnel::thread_proc(
                    tun_file.as_mut().expect("can not found tun file"),
                    acc_file.as_mut().expect("can not foun acc file"),
                    handle_type,
                );
            })));
        }
    }

    pub fn stop(&mut self) {
        self.is_started.store(false, atomic::Ordering::SeqCst);

        for handle in &mut self.handles {
            if let Some(handle) = handle.take() {
                handle.join().unwrap();
            }
        }
    }

    pub fn is_started(&self) -> bool {
        self.is_started.load(atomic::Ordering::SeqCst)
    }
}
