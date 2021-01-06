use nmea::Nmea;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let port_name = "/dev/ttyACM0";
    let baudrate = 115_200;
    let mut nmea = Nmea::new();
    let (mut lat, mut lon) = (0.0, 0.0);
    let (tx, rx) = mpsc::channel();

    let gps = thread::spawn(move || {
        let mut port = serialport::new(port_name, baudrate)
            .timeout(Duration::from_millis(100))
            .open()
            .expect(&format!(
                "Error opening port {} with baudrate {}",
                port_name, baudrate
            ));

        let mut serial_buf: Vec<u8> = vec![0; 1000];
        loop {
            match port.read(serial_buf.as_mut_slice()) {
                Ok(t) => {
                    // println!("Length of data from port is {}", t);
                    let sentence = std::str::from_utf8(&serial_buf[..t]).unwrap();
                    let phrases: Vec<&str> = sentence.split("\r\n").collect();
                    // println!("phrases:\n{:?}", phrases);

                    for phrase in &phrases {
                        // println!("PHRASE: {}",phrase);
                        match nmea.parse(phrase) {
                            Ok(parsed) => {
                                // println!("Successfully parsed to: {:?} ", nmea.latitude);
                                // println!("(lat, long): ({}, {})",nmea.latitude.unwrap(), nmea.longitude.unwrap());

                                lat = match nmea.latitude {
                                    Some(val) => val,
                                    _ => lat,
                                };
                                lon = match nmea.longitude {
                                    Some(val) => val,
                                    _ => lon,
                                };
                                tx.send((lat, lon)).unwrap();
                            }
                            Err(e) => {
                                // println!("ERROR PARSING")
                                ()
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
        }
    });

    let eps = 1e-6;
    let (mut lat, mut lon) = (0.0, 0.0);
    loop {
        match rx.recv() {
            Ok((a, b)) => {
                if ((lat - a) > eps) {
                    lat = a;
                    println!("(lat, lon) = ({}, {})", lat, lon);
                } 
                if ((lon - b) > eps) {
                    lon = b;
                    println!("(lat, lon) = ({}, {})", lat, lon);
                } 
                lat = a;
                lon = b;
            }
            Err(e) => (),
        };
        println!("(lat, lon) = ({}, {})", lat, lon);
        thread::sleep(Duration::from_millis(2000));
    }

    gps.join().unwrap();
}
