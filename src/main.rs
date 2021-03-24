use nav_types::*;
use nmea::Nmea;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let port_name = "/dev/ttyACM0";
    let baudrate = 115_200;
    let mut nmea = Nmea::new();
    let (mut lat, mut lon) = (0.0, 0.0);
    let (tx_gps, rx_gps) = mpsc::channel();

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
                            Ok(_) => {
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
                                tx_gps.send((lat, lon)).unwrap();
                            }
                            Err(_) => {
                                //println!("{:?}",e)
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

    for _ in 0..10 {
        match rx_gps.try_recv() {
            Ok((a, b)) => {
                if lat - a > eps {
                    lat = a;
                    println!("(lat, lon) = ({}, {})", lat, lon);
                }
                if lon - b > eps {
                    lon = b;
                    println!("(lat, lon) = ({}, {})", lat, lon);
                }
                lat = a;
                lon = b;
            }
            Err(_) => (),
        };
        let actual = WGS84::from_degrees_and_meters(lat, lon, 0.0);
        println!(
            "Current position: ({},{})",
            actual.latitude_degrees(),
            actual.longitude_degrees()
        );
        let point = WGS84::from_degrees_and_meters(34.404507405298624, -119.84407870487519, 0.0);
        let pier = WGS84::from_degrees_and_meters(34.41296949371489, -119.82826439753913, 0.0);

        let point_distance = actual.distance(&point);
        let point_bearing = estimate_bearing(actual, point);
        println!(
            "The Point is {:.2} m away at a bearing of appx. {} degrees",
            point_distance, point_bearing
        );

        let pier_distance = actual.distance(&pier);
        let pier_bearing = estimate_bearing(actual, pier);
        println!(
            "The Pier is {:.2} m away at a bearing of appx. {} degrees",
            pier_distance, pier_bearing
        );

        thread::sleep(Duration::from_millis(1000));
    }

    gps.join().unwrap();
}

// Based on `rust-navigation` crate
use std::f64::consts::PI;
fn estimate_bearing(a: WGS84<f64>, b: WGS84<f64>) -> f64 {
    let start_lat = a.latitude_radians();
    let start_lon = a.longitude_radians();
    let dest_lat = b.latitude_radians();
    let dest_lon = b.longitude_radians();

    let mut delta_lon = dest_lon - start_lon;

    let delta_phi = ((dest_lat / 2.0 + PI / 4.0).tan() / (start_lat / 2.0 + PI / 4.0).tan()).ln();

    if delta_lon.abs() > PI {
        if delta_lon > 0.0 {
            delta_lon = -(2.0 * PI - delta_lon);
        } else {
            delta_lon += 2.0 * PI;
        }
    }

    let bearing = (delta_lon.atan2(delta_phi).to_degrees() + 360.0) % 360.0;
    let bearing = match bearing <= 180.0 {
        true => bearing,
        false => -(360.0 - bearing),
    };
    bearing
}
