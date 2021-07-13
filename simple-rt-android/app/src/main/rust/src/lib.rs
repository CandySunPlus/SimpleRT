use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Mutex;
use std::thread;

use android_logger::Config;
use jni::objects::{JClass, JObject};
use jni::sys::{jint, JNI_VERSION_1_8};
use jni::{JNIEnv, JavaVM};
use lazy_static::lazy_static;
use log::{trace, Level, error};
use stdext::function_name;

const TAG: &str = "TetherService";

struct Module {
    tun_handle: Option<thread::JoinHandle<()>>,
    acc_handle: Option<thread::JoinHandle<()>>,
    tun_fd: i32,
    acc_fd: i32,
    is_started: sync::Arc<AtomicBool>,
}

lazy_static! {
    static ref MODULE: Mutex<Module> = Mutex::new(None);
}

impl Module {
    pub fn new(tun_fd: i32, acc_fd: i32) -> Self {
        Self {
            tun_handle: None,
            acc_handle: None,
            tun_fd,
            acc_fd,
            is_started: sync::Arc::new(AtomicBool::new(false)),
        }
    }
}

#[no_mangle]
pub extern "C" fn JNI_OnLoad(_jvm: JavaVM, _reserved: *mut c_void) -> jint {
    android_logger::init_once(Config::default().with_min_level(Level::Trace).with_tag(TAG));
    trace!(function_name! {});
    JNI_VERSION_1_8
}

#[no_mangle]
pub extern "C" fn Java_com_viper_simplert_Native_start(
    env: JNIEnv,
    _class: JClass,
    tun_fd: jint,
    acc_fd: jint,
    callback: JObject,
) {
    trace!(
        "{}: tun_fd = {}, acc_fd = {}",
        function_name!(),
        tun_fd,
        acc_fd
    );

    if (MODULE.lock().unwrap().is_started) {
        error!("Native threads already started!");
        return;
    }
}
