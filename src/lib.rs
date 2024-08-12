// Jackson Coxson

use std::{
    net::SocketAddrV4,
    str::FromStr,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
};

use boringtun::crypto::{X25519PublicKey, X25519SecretKey};
use log::error;
use once_cell::sync::Lazy;

static GLOBAL_HANDLE: Lazy<Mutex<Option<Sender<()>>>> = Lazy::new(|| Mutex::new(None));

#[swift_bridge::bridge]
mod ffi {
    #[derive(Debug)]
    #[swift_bridge(swift_name = "EMPError")]
    pub enum Error {
        InvalidAddress,
        InvalidPort,
        InvalidSocket,
        AlreadyRunning,
        Unknown,
    }
    extern "Rust" {
        fn start_emotional_damage(bind_addr: &str) -> Option<Error>;
        fn stop_emotional_damage();
        fn test_emotional_damage(timeout: u32) -> Option<Error>;
    }
}
use ffi::Error;

pub fn start_loopback(bind_addr: SocketAddrV4) -> Sender<()> {
    // Create the handle
    let (tx, rx) = channel();

    // Read the keys to memory
    let server_private = include_str!("../keys/server_privatekey")[..44].to_string();
    let client_public = include_str!("../keys/client_publickey")[..44].to_string();

    let server_private = X25519SecretKey::from_str(&server_private).unwrap();
    let client_public = X25519PublicKey::from_str(&client_public).unwrap();

    let tun = boringtun::noise::Tunn::new(
        Arc::new(server_private),
        Arc::new(client_public),
        None,
        None,
        0,
        None,
    )
    .unwrap();

    std::thread::spawn(move || {
        // Try and wait for the socket to become available
        let mut socket;
        loop {
            match std::net::UdpSocket::bind(bind_addr) {
                Ok(s) => socket = s,
                Err(e) => match e.kind() {
                    std::io::ErrorKind::AddrInUse => {
                        println!("EMP address in use, retrying...");
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        continue;
                    }
                    _ => panic!(),
                },
            };
            break;
        }

        let mut ready = false;
        loop {
            // Attempt to read from the UDP socket
            match socket.set_read_timeout(Some(std::time::Duration::from_millis(5))) {
                Ok(_) => {}
                Err(e) => {
                    println!("Unable to set UDP timeout: {:?}\nRebinding to socket", e);
                    std::mem::drop(socket);

                    // Wait until we can rebind to the socket
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        socket = match std::net::UdpSocket::bind(bind_addr) {
                            Ok(s) => s,
                            Err(e) => {
                                println!("Socket not dropped: {:?}", e);
                                continue;
                            }
                        };
                        println!("Rebound to socket!");
                        break;
                    }
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
                                println!("Ready!!");
                            }
                        }
                        boringtun::noise::TunnResult::Err(_) => {
                            // don't care
                        }
                        boringtun::noise::TunnResult::WriteToNetwork(b) => {
                            socket.send_to(b, endpoint).unwrap();
                            loop {
                                let p =
                                    tun.decapsulate(Some(endpoint.ip()), &[], &mut unencrypted_buf);
                                match p {
                                    boringtun::noise::TunnResult::WriteToNetwork(b) => {
                                        socket.send_to(b, endpoint).unwrap();
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
                                    socket.send_to(b, endpoint).unwrap();
                                }
                                _ => {
                                    println!("Unexpected result");
                                }
                            }
                        }
                        boringtun::noise::TunnResult::WriteToTunnelV6(_b, _addr) => {
                            panic!("IPv6 not supported");
                        }
                    }
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::WouldBlock => {}
                    std::io::ErrorKind::TimedOut => {}
                    _ => {
                        error!("Error receiving: {}", e);
                        return;
                    }
                },
            }
            // Die if instructed or if the handle was destroyed
            match rx.try_recv() {
                Ok(_) => {
                    println!("EMP instructed to die");
                    return;
                }
                Err(e) => match e {
                    std::sync::mpsc::TryRecvError::Empty => continue,
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        println!("Handle has been destroyed");
                        return;
                    }
                },
            }
        }
    });

    tx
}

/// Starts your emotional damage
/// # Arguments
/// * `bind_addr` - The UDP socket to listen to
/// # Returns
/// A handle to stop further emotional damage.
/// Null on failure
/// # Safety
/// Don't be stupid
fn start_emotional_damage(bind_addr: &str) -> Option<Error> {
    // Check if the proxy exists
    if GLOBAL_HANDLE.lock().unwrap().is_some() {
        println!("Proxy already exists, skipping");
        return Some(Error::AlreadyRunning);
    }
    let address = match bind_addr.parse::<SocketAddrV4>() {
        Ok(address) => address,
        Err(_) => return Some(Error::InvalidAddress),
    };
    let handle = start_loopback(address);

    *GLOBAL_HANDLE.lock().unwrap() = Some(handle);

    None
}

/// Stops further emotional damage
/// # Arguments
/// * `handle` - The coping mechanism generated by start_emotional_damage
/// # Returns
/// The knowledge of knowing that you couldn't handle failure
/// # Safety
/// Don't be stupid
fn stop_emotional_damage() {
    let sender = GLOBAL_HANDLE.lock().unwrap().clone();
    if let Some(sender) = sender {
        if sender.send(()).is_ok() {
            //
        }
    }
    *GLOBAL_HANDLE.lock().unwrap() = None;
}

/// Blocks until Wireguard is ready
/// # Arguments
/// * `timeout` - The timeout in miliseconds to wait for Wireguard
/// # Returns
/// 0 on success, -1 on failure
fn test_emotional_damage(timeout: u32) -> Option<Error> {
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
                    return Some(Error::InvalidSocket);
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
        Ok(_) => None,
        Err(e) => {
            println!("Never received test data: {:?}", e);
            Some(Error::Unknown)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::Ipv4Addr,
        str::FromStr,
    };

    use super::*;

    #[test]
    fn pls_yeet() {
        let bind_addr = SocketAddrV4::from_str("127.0.0.1:51820").unwrap();
        let _handle = start_loopback(bind_addr);

        let num = 100;
        let size = 100_000;

        // Create TCP listener
        let listener = std::net::TcpListener::bind("0.0.0.0:3000").unwrap();
        let (send_ready, ready) = channel();

        // A place to store the test data
        let tests = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let spawn_tests = tests.clone();

        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();

            // Create test data
            let mut local_tests = Vec::new();
            for _ in 0..num {
                let mut test = Vec::new();
                for _ in 0..size {
                    test.push(rand::random::<u8>());
                }
                tests.lock().unwrap().extend(test.clone());
                local_tests.push(test);
            }

            // Wait until we're ready to send the test
            ready.recv().unwrap();

            // Send the test data
            for test in local_tests {
                stream.write_all(&test).unwrap();
                std::thread::sleep(std::time::Duration::from_nanos(1));
            }
        });

        let mut connector =
            std::net::TcpStream::connect(SocketAddrV4::new(Ipv4Addr::new(10, 7, 0, 1), 3000))
                .unwrap();
        send_ready.send(()).unwrap();

        // Collect the test data
        let mut collected_tests: Vec<u8> = Vec::new();

        let current_time = std::time::Instant::now();

        loop {
            let mut buf = [0_u8; 2048];
            match connector.read(&mut buf) {
                Ok(size) => {
                    if size == 0 {
                        break;
                    }
                    let buf = &buf[0..size];
                    collected_tests.extend(buf);
                }
                Err(_e) => {
                    break;
                }
            }
        }

        println!("Elapsed time: {:?}", current_time.elapsed());
        println!(
            "MB/s: {:?}",
            collected_tests.len() as f64 / current_time.elapsed().as_secs_f64()
        );

        // Compare the two
        println!("Testing length");
        assert_eq!(collected_tests.len(), spawn_tests.lock().unwrap().len());
        println!("Testing contents");
        assert_eq!(collected_tests, spawn_tests.lock().unwrap()[..]);

        println!("All tests passed");
    }
}
