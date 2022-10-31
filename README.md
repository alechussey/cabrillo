# Cabrillo

A Rust / nom parser for the Cabrillo file format used for submitting contest logs in amateur radio.

## Example

```rust
use std::fs;
use cabrillo::CabrilloLog;

fn main() {
	let buf = fs::read("mylog.txt").unwrap();
	let log = CabrilloLog::from_buffer(&buf).unwrap();

	println!("{:#?}", log);
}
```

```
CabrilloLog {
    version: 3.0,
    callsign: Some(
        "AA1ZZZ",
    ),
    contest: Some(
        "CQ-WW-SSB",
    ),
    category_assisted: Some(
        false,
    ),
    category_band: Some(
        All,
    ),
    category_mode: Some(
        Phone,
    ),
    category_operator: Some(
        SingleOp,
    ),
    category_power: Some(
        High,
    ),
    category_station: None,
    category_time: None,
    category_transmitter: Some(
        One,
    ),
    category_overlay: Some(
        Classic,
    ),
    certificate: None,
    claimed_score: Some(
        9447852,
    ),
    club: Some(
        "Yankee Clipper Contest Club",
    ),
    created_by: Some(
        "WriteLog V10.72C",
    ),
    email: None,
    grid_locator: Some(
        "FN20ib",
    ),
    location: Some(
        "WMA",
    ),
    name: Some(
        "Randy Thompson",
    ),
    address: Some(
        "1 Main St\nUxbridge\nMA\n01569\nUSA",
    ),
    operators: [
        "K5ZD",
    ],
    offtimes: [],
    soapbox: Some(
        "Put your comments here.\nUse multiple lines if needed.",
    ),
    other_tags: {},
    entries: [
        Qso {
            frequency: Khz(
                3799,
            ),
            mode: Phone,
            datetime: 2000-10-26T07:11:00,
            call_sent: "AA1ZZZ",
            exch_sent: "59 05",
            call_recvd: "K9QZO",
            exch_recvd: "59 04",
            transmitter_id: false,
        },
        Qso {
            frequency: Khz(
                14256,
            ),
            mode: Phone,
            datetime: 2000-10-26T07:11:00,
            call_sent: "AA1ZZZ",
            exch_sent: "59 05",
            call_recvd: "P29AS",
            exch_recvd: "59 28",
            transmitter_id: false,
        },
        Qso {
            frequency: Khz(
                21250,
            ),
            mode: Phone,
            datetime: 2000-10-26T07:11:00,
            call_sent: "AA1ZZZ",
            exch_sent: "59 05",
            call_recvd: "4S7TWG",
            exch_recvd: "59 22",
            transmitter_id: false,
        },
        Qso {
            frequency: Khz(
                28530,
            ),
            mode: Phone,
            datetime: 2000-10-26T07:11:00,
            call_sent: "AA1ZZZ",
            exch_sent: "59 05",
            call_recvd: "JT1FAX",
            exch_recvd: "59 23",
            transmitter_id: false,
        },
        Qso {
            frequency: Khz(
                7250,
            ),
            mode: Phone,
            datetime: 2000-10-26T07:11:00,
            call_sent: "AA1ZZZ",
            exch_sent: "59 05",
            call_recvd: "WA6MIC",
            exch_recvd: "59 03",
            transmitter_id: false,
        },
    ],
    ignored_entries: [],
    debug: false,
}
```
