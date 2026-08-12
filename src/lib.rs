// Jackson Coxson

use std::{
    ffi::CStr,
    net::SocketAddrV4,
    os::raw::{c_char, c_int},
    str::FromStr,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
};
use boringtun::crypto::{X25519PublicKey, X25519SecretKey};
use log::error;
use once_cell::sync::Lazy;

pub type LogCallbackFn = unsafe extern "C" fn(level: c_int, msg: *const c_char) -> bool;

struct ProxyHandle {
    sender: Sender<()>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}

static GLOBAL_HANDLE: Lazy<Mutex<Option<ProxyHandle>>> = Lazy::new(|| Mutex::new(None));
static LOG_CALLBACK: Lazy<Mutex<Option<LogCallbackFn>>> = Lazy::new(|| Mutex::new(None));

#[no_mangle]
pub unsafe extern "C" fn set_log_callback(cb: Option<LogCallbackFn>) {
    let mut lock = LOG_CALLBACK.lock().unwrap_or_else(|e| e.into_inner());
    *lock = cb;
}

fn log_msg(level: c_int, msg: String) {
    let cb = {
        let lock = LOG_CALLBACK.lock().unwrap_or_else(|e| e.into_inner());
        *lock
    };

    if let Some(cb) = cb {
        if let Ok(c_msg) = std::ffi::CString::new(msg.clone()) {
            unsafe {
                if cb(level, c_msg.as_ptr()) {
                    return;
                }
            }
        }
    }

    match level {
        0 => println!("[DEBUG] {}", msg),
        1 => println!("[INFO] {}", msg),
        2 => println!("[WARN] {}", msg),
        _ => error!("{}", msg),
    }
}

macro_rules! base_path {
    () => {
        "../keys"
    };
}

pub fn start_loopback(bind_addr: SocketAddrV4) -> Result<ProxyHandle, c_int> {
    // Create the handle
    let (tx, rx) = channel();

    // Read the keys to memory
    let server_private = include_str!(concat!(base_path!(), "/server_privatekey"))[..44].to_string();
    let client_public = include_str!(concat!(base_path!(), "/client_publickey"))[..44].to_string();

    let server_private = match X25519SecretKey::from_str(&server_private) {
        Ok(k) => k,
        Err(e) => {
            log_msg(3, format!("Failed to parse server private key: {:?}", e));
            return Err(-5);
        }
    };
    let client_public = match X25519PublicKey::from_str(&client_public) {
        Ok(k) => k,
        Err(e) => {
            log_msg(3, format!("Failed to parse client public key: {:?}", e));
            return Err(-5);
        }
    };

    let tun = match boringtun::noise::Tunn::new(
        Arc::new(server_private),
        Arc::new(client_public),
        None,
        None,
        0,
        None,
    ) {
        Ok(t) => t,
        Err(e) => {
            log_msg(3, format!("Failed to initialize boringtun Tunn: {:?}", e));
            return Err(-5);
        }
    };

    // Synchronously bind socket before returning to caller
    let socket = match std::net::UdpSocket::bind(bind_addr) {
        Ok(s) => s,
        Err(e) => {
            log_msg(3, format!("EMP socket bind error: {:?}", e));
            return Err(-4);
        }
    };

    let join_handle = std::thread::spawn(move || {
        let mut ready = false;
        loop {
            // Attempt to read from the UDP socket
            match socket.set_read_timeout(Some(std::time::Duration::from_millis(5))) {
                Ok(_) => {}
                Err(e) => {
                    log_msg(2, format!("Unable to set UDP timeout: {:?}\nRebinding to socket", e));
                    continue;
                }
            }
            let mut buf = [0_u8; 2048]; // we can use a small buffer because it will tell us if more is needed
            match socket.recv_from(&mut buf) {
                Ok((size, endpoint)) => {
                    // Parse it with boringtun
                    let mut unencrypted_buf = [0; 2176];
                    let p =
                        tun.decapsulate(Some(endpoint.ip()), &buf[..size], &mut unencrypted_buf);

                    match p {
                        boringtun::noise::TunnResult::Done => {
                            // literally nobody knows what to do with this
                            if !ready {
                                ready = true;
                                log_msg(1, "Ready!!".to_string());
                            }
                        }
                        boringtun::noise::TunnResult::Err(_) => {
                            // don't care
                        }
                        boringtun::noise::TunnResult::WriteToNetwork(b) => {
                            if let Err(e) = socket.send_to(b, endpoint) {
                                log_msg(3, format!("Error sending UDP packet: {:?}", e));
                            }
                            loop {
                                let p =
                                    tun.decapsulate(Some(endpoint.ip()), &[], &mut unencrypted_buf);
                                match p {
                                    boringtun::noise::TunnResult::WriteToNetwork(b) => {
                                        if let Err(e) = socket.send_to(b, endpoint) {
                                            log_msg(3, format!("Error sending UDP packet: {:?}", e));
                                        }
                                    }
                                    _ => break,
                                }
                            }
                        }
                        boringtun::noise::TunnResult::WriteToTunnelV4(b, _addr) => {
                            // Swap bytes 12-15 with 16-19
                            b.swap(12, 16);
                            b.swap(13, 17);
                            b.swap(14, 18);
                            b.swap(15, 19);

                            let mut buf = [0_u8; 2048];
                            match tun.encapsulate(b, &mut buf) {
                                boringtun::noise::TunnResult::WriteToNetwork(b) => {
                                    if let Err(e) = socket.send_to(b, endpoint) {
                                        log_msg(3, format!("Error sending UDP packet: {:?}", e));
                                    }
                                }
                                _ => {
                                    log_msg(2, "Unexpected result".to_string());
                                }
                            }
                        }
                        boringtun::noise::TunnResult::WriteToTunnelV6(_b, _addr) => {
                            log_msg(2, "IPv6 packet ignored".to_string());
                        }
                    }
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::WouldBlock => {}
                    std::io::ErrorKind::TimedOut => {}
                    _ => {
                        log_msg(3, format!("Error receiving: {}", e));
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                },
            }
            // Die if instructed or if the handle was destroyed
            match rx.try_recv() {
                Ok(_) => {
                    log_msg(1, "EMP instructed to die".to_string());
                    return;
                }
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => continue,
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        log_msg(1, "Handle has been destroyed".to_string());
                        return;
                    }
                },
            }
        }
    });

    Ok(ProxyHandle {
        sender: tx,
        join_handle: Some(join_handle),
    })
}

#[no_mangle]
/// Starts your emotional damage
/// # Arguments
/// * `bind_addr` - The UDP socket to listen to
/// # Returns
/// 0 on success, -1 if null address, -2 if UTF-8 error, -3 if invalid socket address, -4 if socket bind failed, -5 if crypto init failed
/// # Safety
/// Don't be stupid
pub unsafe extern "C" fn start_emotional_damage(bind_addr: *const c_char) -> c_int {
    let mut handle_lock = GLOBAL_HANDLE.lock().unwrap_or_else(|e| e.into_inner());
    // Check if the proxy exists
    if handle_lock.is_some() {
        log_msg(1, "Proxy already exists, skipping".to_string());
        return 0;
    }
    // Check the address
    if bind_addr.is_null() {
        return -1;
    }
    let address = CStr::from_ptr(bind_addr as *mut _);
    let address = match address.to_str() {
        Ok(address) => address,
        Err(_) => return -2,
    };
    let address = match address.parse::<SocketAddrV4>() {
        Ok(address) => address,
        Err(_) => return -3,
    };
    let handle = match start_loopback(address) {
        Ok(h) => h,
        Err(err) => return err,
    };

    *handle_lock = Some(handle);

    0
}

#[no_mangle]
/// Stops further emotional damage
/// # Returns
/// 0 on success, -1 if no server running, -2 if failed to send stop signal, -3 if thread join failed
/// # Safety
/// Don't be stupid
pub unsafe extern "C" fn stop_emotional_damage() -> c_int {
    let mut handle_lock = GLOBAL_HANDLE.lock().unwrap_or_else(|e| e.into_inner());
    match handle_lock.take() {
        Some(handle) => {
            let send_res = handle.sender.send(());
            let join_res = if let Some(jh) = handle.join_handle {
                jh.join()
            } else {
                Ok(())
            };
            match (send_res, join_res) {
                (Ok(()), Ok(())) => 0,
                (Err(_), _) => -2,
                (_, Err(_)) => -3,
            }
        }
        None => -1,
    }
}

#[no_mangle]
/// Blocks until Wireguard is ready
/// # Arguments
/// * `timeout` - The timeout in miliseconds to wait for Wireguard
/// # Returns
/// 0 on success, -1 on failure
pub extern "C" fn test_emotional_damage(timeout: c_int) -> c_int {
    // Bind to the testing socket ASAP
    let mut testing_port = 3000_u16;
    let listener;
    loop {
        listener = match std::net::UdpSocket::bind((
            std::net::Ipv4Addr::new(127, 0, 0, 1),
            testing_port,
        )) {
            Ok(l) => l,
            Err(e) => match e.kind() {
                std::io::ErrorKind::AddrInUse => {
                    testing_port += 1;
                    continue;
                }
                _ => {
                    println!("Unable to bind to UDP socket");
                    return -1;
                }
            },
        };
        break;
    }
    listener
        .set_read_timeout(Some(std::time::Duration::from_millis(timeout as u64)))
        .unwrap();

    std::thread::spawn(move || {
        let sender =
            std::net::UdpSocket::bind((std::net::Ipv4Addr::new(127, 0, 0, 1), testing_port + 1))
                .unwrap();
        for _ in 0..10 {
            if sender
                .send_to(&[69], (std::net::Ipv4Addr::new(127, 0, 0, 1), testing_port))
                .is_ok()
            {
                // pass
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });

    let mut buf = [0_u8; 1];
    match listener.recv(&mut buf) {
        Ok(_) => 0,
        Err(e) => {
            println!("Never received test data: {:?}", e);
            -1
        }
    }
}
