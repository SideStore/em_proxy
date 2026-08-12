// Jackson Coxson
// Moved from src/lib.rs to tests/test_proxy.rs

use em_proxy::*;
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
    sync::mpsc::channel,
};

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
