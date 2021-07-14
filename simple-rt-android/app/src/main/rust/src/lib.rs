use android_logger::Config;
use jni::objects::JClass;
use jni::sys::{jboolean, jint, JNI_VERSION_1_8};
use jni::{JNIEnv, JavaVM};
use lazy_static::lazy_static;
use log::{error, trace, Level};
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use crate::tunnel::Tunnel;

mod tunnel;

const TAG: &str = "TetherService";

lazy_static! {
    static ref TUNNEL: Arc<Mutex<tunnel::Tunnel>> = Arc::new(Mutex::new(tunnel::Tunnel::new()));
}

#[no_mangle]
pub extern "C" fn JNI_OnLoad(_jvm: JavaVM, _reserved: *mut c_void) -> jint {
    android_logger::init_once(Config::default().with_min_level(Level::Trace).with_tag(TAG));
    trace!("JNI ONLOAD");
    JNI_VERSION_1_8
}

#[no_mangle]
pub extern "C" fn Java_com_viper_simplert_Native_start(
    _env: JNIEnv,
    _class: JClass,
    tun_fd: jint,
    acc_fd: jint,
) {
    trace!("START: tun_fd = {}, acc_fd = {}", tun_fd, acc_fd);
    let tunnel = TUNNEL.lock().unwrap();

    if tunnel.is_started() {
        error!("Native threads already started!");
        return;
    }

    Tunnel::start(TUNNEL.clone(), tun_fd, acc_fd)
}

#[no_mangle]
pub extern "C" fn Java_com_viper_simplert_Native_stop(_env: JNIEnv, _class: JClass) {
    trace!("STOP");
    let mut tunnel = TUNNEL.lock().unwrap();
    tunnel.stop();
}

#[no_mangle]
pub extern "C" fn Java_com_viper_simplert_Native_is_running(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    trace!("CHECK RUNNING");
    let tunnel = TUNNEL.lock().unwrap();
    tunnel.is_started().into()
}
