use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::prelude::{FromRawFd, RawFd};
use std::sync::{atomic, Arc, Mutex};
use std::thread;
use log::trace;

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

enum HandleType {
    Tun,
    Acc,
}

pub struct Tunnel {
    handles: Vec<Option<thread::JoinHandle<()>>>,
    tun_file: Option<File>,
    acc_file: Option<File>,
    is_started: Arc<atomic::AtomicBool>,
}

const ACC_BUF_SIZE: usize = 4096;

impl Tunnel {
    pub fn new() -> Self {
        Self {
            handles: vec![],
            tun_file: None,
            acc_file: None,
            is_started: Arc::new(atomic::AtomicBool::new(false)),
        }
    }

    fn set_fd(&mut self, tun_fd: RawFd, acc_fd: RawFd) {
        if let Ok(flags) = syscall!(fcntl(acc_fd, libc::F_GETFL, 0)) {
            syscall!(fcntl(acc_fd, libc::F_SETFL, flags & !libc::O_NONBLOCK))
                .expect("set acc_fd flags failed");
            self.acc_file = Some(unsafe { File::from_raw_fd(acc_fd) });
        }

        if let Ok(flags) = syscall!(fcntl(tun_fd, libc::F_GETFL, 0)) {
            syscall!(fcntl(tun_fd, libc::F_SETFL, flags & !libc::O_NONBLOCK))
                .expect("set tun_fd flags failed");
            self.tun_file = Some(unsafe { File::from_raw_fd(tun_fd) });
        }
    }

    fn thread_proc(tunnel: Arc<Mutex<Self>>, handle_type: HandleType) {
        let mut buf = [0u8; ACC_BUF_SIZE];
        let mut in_file: File;
        let mut out_file: File;
        trace!("start tunnel thread");
        match handle_type {
            HandleType::Tun => {
                in_file = tunnel
                    .lock()
                    .unwrap()
                    .tun_file
                    .take()
                    .expect("tun file not found");
                out_file = tunnel
                    .lock()
                    .unwrap()
                    .acc_file
                    .take()
                    .expect("acc file not found");
            }
            HandleType::Acc => {
                in_file = tunnel
                    .lock()
                    .unwrap()
                    .acc_file
                    .take()
                    .expect("acc file not found");
                out_file = tunnel
                    .lock()
                    .unwrap()
                    .tun_file
                    .take()
                    .expect("tun file not found");
            }
        }

        while tunnel.lock().unwrap().is_started() {
            if let Ok(_) = in_file.read(&mut buf) {
                out_file.write(&buf).expect("write file error");
            } else {
                break;
            }
        }

        tunnel
            .lock()
            .unwrap()
            .is_started
            .store(false, atomic::Ordering::SeqCst);
    }

    fn init(&mut self, tun_fd: i32, acc_fd: i32) {
        self.is_started.store(true, atomic::Ordering::SeqCst);
        self.set_fd(tun_fd, acc_fd);
    }

    pub fn start(tunnel: Arc<Mutex<Self>>, tun_fd: RawFd, acc_fd: RawFd) {
        tunnel.lock().unwrap().init(tun_fd, acc_fd);

        for handle_type in vec![HandleType::Tun, HandleType::Acc] {
            let local_tunnel = tunnel.clone();
            tunnel
                .lock()
                .unwrap()
                .handles
                .push(Some(thread::spawn(move || {
                    Tunnel::thread_proc(local_tunnel, handle_type);
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
