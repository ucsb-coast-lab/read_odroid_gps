### Parsing GPS Sentences over USB

This is a sketch of program that reads in and parses NMEA sentences from an
[Odroid GPS module](https://ameridroid.com/products/usb-gps-module?_pos=1&_sid=6a8253b25&_ss=r) over USB. The default port for this device is listed at 
`/dev/ttyACM0` with a baudrate of 115200. 

It make take some time for the device to achieve an actual lock, but should 
do so within approximately 15 seconds. Until then, both latitude and longitude
values will read as (0, 0). 

This has been tested on an x86 desktop and an Odroid-C4 with `rustc 1.49-stable`.