use android_logger::Config;
use jni::objects::JClass;
use jni::sys::{jboolean, jint, JNI_VERSION_1_6};
use jni::{JNIEnv, JavaVM};
use lazy_static::lazy_static;
use log::{error, trace, Level};
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

mod tunnel;
use tunnel::Tunnel;

const TAG: &str = "TetherService";

lazy_static! {
    static ref TUNNEL: Arc<Mutex<Tunnel>> = Arc::new(Mutex::new(Tunnel::new()));
}


#[no_mangle]
pub extern "C" fn JNI_OnLoad(_jvm: JavaVM, _reserved: *mut c_void) -> jint {
    if cfg!(debug_assertions) {
        android_logger::init_once(Config::default().with_min_level(Level::Trace).with_tag(TAG));
    } else {
        android_logger::init_once(Config::default().with_min_level(Level::Info).with_tag(TAG));
    }
    trace!("JNI ONLOAD");
    JNI_VERSION_1_6
}

#[no_mangle]
pub extern "C" fn Java_com_viper_simplert_Native_start(
    _env: JNIEnv,
    _class: JClass,
    tun_fd: jint,
    acc_fd: jint,
) {
    trace!("START: tun_fd = {}, acc_fd = {}", tun_fd, acc_fd);

    if TUNNEL.lock().unwrap().is_started() {
        error!("Native threads already started!");
        return;
    }

    trace!("TUNNEL STARTING");
    Tunnel::start(TUNNEL.clone(), tun_fd, acc_fd);
    trace!("TUNNEL STARTED");
}

#[no_mangle]
pub extern "C" fn Java_com_viper_simplert_Native_stop(_env: JNIEnv, _class: JClass) {
    trace!("STOP");
    TUNNEL.lock().unwrap().stop();
}

#[no_mangle]
pub extern "C" fn Java_com_viper_simplert_Native_isRunning(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    trace!("CHECK RUNNING");
    TUNNEL.lock().unwrap().is_started().into()
}
