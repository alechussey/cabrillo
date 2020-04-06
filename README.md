# Cabrillo

A Rust / nom parser for the Cabrillo file format used for submitting contest logs in amateur radio.

## Example

```
use std::fs;
use cabrillo::CabrilloLog;

fn main() {
	let buf = fs::read("mylog.txt").unwrap();
	let log = CabrilloLog::from_buffer(&buf).unwrap();

	println!("{:#?}", log);
}
```
