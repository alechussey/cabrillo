#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nom;
extern crate chrono;

use std::io::BufRead;
use std::str::{self, FromStr};
use std::fmt::{self, Display};
use std::error::Error;
use std::convert::TryFrom;
use std::collections::HashMap;
use chrono::NaiveDateTime;
use nom::character::is_digit;
use nom::character::complete::{
	digit1,
	multispace0,
	alphanumeric1,
	not_line_ending
};

fn is_cabrillo_tag(c: char) -> bool {
	c.is_ascii_uppercase() || is_digit(c as u8) || c == '-' || c == '\'' || c == ' '
}

named!(
	cabrillo_tag<&str, (&str, &str)>,
	alt!(
		complete!(
			separated_pair!(
				take_while1!(is_cabrillo_tag), 
				tag!(": "), // the spec requires a space after the colon
				not_line_ending
			)
		) |
		map!(tag!("END-OF-LOG"), |s| (s, ""))
	)
);

named!(
	cabrillo_email<&str, &str>,
	re_match_static!(r"^([a-zA-Z0-9_\-\.]+)@([a-zA-Z0-9_\-\.]+)\.([a-zA-Z]{2,5})$")
);

named!(
	cabrillo_grid_locator<&str, &str>,
	re_match_static!(r"^([A-Ra-r]{2})([0-9]{2})([A-Ra-r]{2}){0,1}$")
);

named!(
	cabrillo_callsign<&str, &str>,
	re_find_static!(r"((@{0,1})([A-Za-z0-9]{3,8})(/[A-Za-z0-9]{1,8}){0,1})")
);

named!(
	cabrillo_datetime<&str, &str>,
	re_find_static!(r"([0-9]{4})-([0-9]{2})-([0-9]{2}) ([0-9]{4})")
);

named!(
	cabrillo_mode<&str, &str>,
	alt!(tag!("CW") | tag!("PH") | tag!("FM")| tag!("RY") | tag!("DG"))
);

named!(
	cabrillo_offtime<&str, (&str, &str)>,
	separated_pair!(
		cabrillo_datetime,
		char!(' '),
		cabrillo_datetime
	)
);

// The only nice way to deal with minor differences in the QSO format is to have
// separate parsers. If you don't do this then you inevitably end up down a rabbit
// hole full of regressions and undefined behavior.

named!(
	cabrillo_qso_format1<&str, (&str, &str, &str, &str, &str, &str, &str, &str, &str)>,
	sep!(
		multispace0,
		tuple!(
			digit1,             // Frequency
			cabrillo_mode,      // Mode
			cabrillo_datetime,  // QSO timestamp
			cabrillo_callsign,  // Sent call
			alphanumeric1,      // Sent RST or exchange
			alphanumeric1,      // Sent exchange
			cabrillo_callsign,  // Rcvd call
			alphanumeric1,      // Rcvd RST or exchange
			alphanumeric1       // Rcvd exchange
		)
	)
);

named!(
	cabrillo_qso_format2<&str, (&str, &str, &str, &str, &str, &str, &str)>,
	sep!(
		multispace0,
		tuple!(
			digit1,             // Frequency
			cabrillo_mode,      // Mode
			cabrillo_datetime,  // QSO timestamp
			cabrillo_callsign,  // Sent call
			alphanumeric1,      // Sent exchange
			cabrillo_callsign,  // Rcvd call
			alphanumeric1       // Rcvd exchange
		)
	)
);

#[derive(Debug, Clone)]
pub enum CabrilloErrorKind {
	IoError(String),
	ParseError(String),
	Other(String)
}

impl Display for CabrilloErrorKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			CabrilloErrorKind::IoError(error) => write!(f, "I/O Error: {}", error),
			CabrilloErrorKind::ParseError(error) => write!(f, "Parse Error: {}", error),
			CabrilloErrorKind::Other(error) => write!(f, "Unknown Error: {}", error)
		}
	}
}

#[derive(Debug, Clone)]
pub struct CabrilloError {
	tag: String,
	line: usize,
	kind: CabrilloErrorKind
}

impl CabrilloError {
	pub fn new(tag: &str, line: usize, kind: CabrilloErrorKind) -> Self {
		Self {
			tag: tag.to_string(),
			line: line,
			kind: kind
		}
	}

	pub fn tag(&self) -> &String {
		&self.tag
	}

	pub fn line(&self) -> usize {
		self.line
	}

	pub fn kind(&self) -> &CabrilloErrorKind {
		&self.kind
	}
}

impl Display for CabrilloError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} in tag '{}' on line {}", self.kind, self.tag, self.line)
	}
}

impl Error for CabrilloError {}

pub type CabrilloResult<T> = std::result::Result<T, CabrilloError>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Band {
	All,
	Band160M,
	Band80M,
	Band40M,
	Band20M,
	Band15M,
	Band10M,
	Band6M,
	Band4M,
	Band2M,
	Band222,
	Band432,
	Band902,
	Band1_2G,
	Band2_3G,
	Band3_4G,
	Band5_7G,
	Band10G,
	Band24G,
	Band47G,
	Band75G,
	Band123G,
	Band134G,
	Band241G,
	Light,
	Vhf3Band,
	VhfFmOnly
}

impl FromStr for Band {
	type Err = CabrilloErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"ALL"   => Ok(Band::All),
			"160M"  => Ok(Band::Band160M),
			"80M"   => Ok(Band::Band80M),
			"40M"   => Ok(Band::Band40M),
			"20M"   => Ok(Band::Band20M),
			"15M"   => Ok(Band::Band15M),
			"10M"   => Ok(Band::Band10M),
			"6M"    => Ok(Band::Band6M),
			"4M"    => Ok(Band::Band4M),
			"2M"    => Ok(Band::Band2M),
			"222"   => Ok(Band::Band222),
			"432"   => Ok(Band::Band432),
			"902"   => Ok(Band::Band902),
			"1.2G"  => Ok(Band::Band1_2G),
			"2.3G"  => Ok(Band::Band2_3G),
			"3.4G"  => Ok(Band::Band3_4G),
			"5.7G"  => Ok(Band::Band5_7G),
			"10G"   => Ok(Band::Band10G),
			"24G"   => Ok(Band::Band24G),
			"47G"   => Ok(Band::Band47G),
			"75G"   => Ok(Band::Band75G),
			"123G"  => Ok(Band::Band123G),
			"134G"  => Ok(Band::Band134G),
			"241G"  => Ok(Band::Band241G),
			"LIGHT" => Ok(Band::Light),
			"VHF-3-BAND" => Ok(Band::Vhf3Band),
			"VHF-FM-ONLY" => Ok(Band::VhfFmOnly),
			_ => Err(
				CabrilloErrorKind::ParseError(format!("Invalid value '{}'", s))
			)
		}
	}
}

impl TryFrom<Frequency> for Band {
	type Error = CabrilloErrorKind;

	fn try_from(other: Frequency) -> Result<Self, Self::Error> {
		match other {
			Frequency::Khz(freq) => {
				match freq {
					_ if freq >= 1800 && freq <= 2000 => Ok(Band::Band160M),
					_ if freq >= 3500 && freq <= 4000 => Ok(Band::Band80M),
					_ if freq >= 7000 && freq <= 7300 => Ok(Band::Band40M),
					_ if freq >= 14000 && freq <= 14350 => Ok(Band::Band20M),
					_ if freq >= 21000 && freq <= 21450 => Ok(Band::Band15M),
					_ if freq >= 28000 && freq <= 29700 => Ok(Band::Band10M),
					_ if freq >= 50000 && freq <= 54000 => Ok(Band::Band6M),
					_ if freq >= 70000 && freq <= 70500 => Ok(Band::Band4M),
					_ if freq >= 144000 && freq <= 148000 => Ok(Band::Band2M),
					_ if freq >= 219000 && freq <= 225000 => Ok(Band::Band222),
					_ if freq >= 420000 && freq <= 450000 => Ok(Band::Band432),
					_ if freq >= 902000 && freq <= 928000 => Ok(Band::Band902),
					_ if freq >= 1240000 && freq <= 1300000 => Ok(Band::Band1_2G),
					_ if freq >= 2390000 && freq <= 2450000 => Ok(Band::Band2_3G),
					_ if freq >= 3300000 && freq <= 3500000 => Ok(Band::Band3_4G),
					_ if freq >= 5650000 && freq <= 5925000 => Ok(Band::Band5_7G),
					_ if freq >= 10000000 && freq <= 10500000 => Ok(Band::Band10G),
					_ if freq >= 24000000 && freq <= 24250000 => Ok(Band::Band24G),
					_ if freq >= 47000000 && freq <= 47200000 => Ok(Band::Band47G),
					_ if freq >= 76000000 && freq <= 81000000 => Ok(Band::Band75G),
					_ if freq >= 122250000 && freq <= 123000000 => Ok(Band::Band123G),
					_ if freq >= 134000000 && freq <= 141000000 => Ok(Band::Band134G),
					_ if freq >= 241000000 && freq <= 250000000 => Ok(Band::Band241G),
					_ if freq >= 300000000 => Ok(Band::Light), // FIXME: I'm not sure what the spec considers to be light
					_ => Err(
						CabrilloErrorKind::ParseError(format!("The value '{}' does not fall within a valid amateur band", other.to_string()))
					)
				}
			}
			Frequency::Light => Ok(Band::Light)
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Frequency {
	Khz(u32),
	Light
}

impl Frequency {
	/// Convert the inner frequency from KHz to MHz. If the frequency is considered 
	/// Light, then None will be returned.
	pub fn as_mhz(&self) -> Option<f32> {
		match self {
			Frequency::Khz(frequency) => {
				let as_mhz: f32 = *frequency as f32 / 1000.0;
				Some(as_mhz)
			},
			Frequency::Light => None
		}
	}

	/// Convert the inner frequency from KHz to GHz. If the frequency is considered
	/// Light, then None will be returned.
	pub fn as_ghz(&self) -> Option<f32> {
		match self {
			Frequency::Khz(frequency) => {
				let as_ghz: f32 = *frequency as f32 / 1000000.0;
				Some(as_ghz)
			},
			Frequency::Light => None
		}
	}

	pub fn is_light(&self) -> bool {
		self == &Frequency::Light
	}
}

impl ToString for Frequency {
	fn to_string(&self) -> String {
		match self {
			Frequency::Khz(freq) => format!("{} KHz", freq),
			Frequency::Light => "LIGHT".to_string()
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mode {
	Cw,
	Phone,
	Fm,
	Rtty,
	Digital,
	Mixed
}

impl FromStr for Mode {
	type Err = CabrilloErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"CW"    => Ok(Mode::Cw),
			"DIGI"  => Ok(Mode::Digital),
			"FM"    => Ok(Mode::Fm),
			"RY"    => Ok(Mode::Rtty),
			"RTTY"  => Ok(Mode::Rtty),
			"PH"    => Ok(Mode::Phone),
			"SSB"   => Ok(Mode::Phone),
			"MIXED" => Ok(Mode::Mixed),
			_ => Err(
				CabrilloErrorKind::ParseError(format!("Invalid value '{}'", s))
			)
		}
	}
}

/// A tuple type representing the 3 parts of a signal report (readability, strength, and tone). If the tone
/// will always be zero if it is not provided.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SignalReport(u8, u8, u8);

impl FromStr for SignalReport {
	type Err = CabrilloErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// all of this nonsense is because from_str_radix requires &str and this is the only way I could think of
		// to get a list of &str's for each character in the string but maybe I'm being dense about his whole thing
		let chars: Vec<&str> = s
			.split("")
			.filter(|i| i != &"")
			.collect();
		
		if chars.len() < 2 || chars.len() > 3 {
			return Err(
				CabrilloErrorKind::ParseError(
					format!("Value has incorrect length ({} bytes); must be 2 or 3.", s.len())
				)
			);
		}

		let parse_error = CabrilloErrorKind::ParseError(format!("Invalid digit in value '{}'", s));
		
		let readability: u8 = u8::from_str_radix(chars[0], 10)
			.map_err(|_| parse_error.clone())?;
		let strength: u8 = u8::from_str_radix(chars[1], 10)
			.map_err(|_| parse_error.clone())?;
		let tone: u8 = if let Some(tone_char) = chars.get(2) {
			u8::from_str_radix(tone_char, 10)
				.map_err(|_| parse_error.clone())?
		} else {
			0
		};

		if readability > 0 && readability <= 5 && strength > 0 && strength <= 9 && tone <= 9 {
			Ok(SignalReport(readability, strength, tone))
		} else {
			Err(parse_error)
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperatorCategory {
	SingleOp,
	MultiOp,
	CheckLog
}

impl FromStr for OperatorCategory {
	type Err = CabrilloErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"SINGLE-OP" => Ok(OperatorCategory::SingleOp),
			"MULTI-OP"  => Ok(OperatorCategory::MultiOp),
			"CHECKLOG"  => Ok(OperatorCategory::CheckLog),
			_ => Err(
				CabrilloErrorKind::ParseError(format!("Invalid value '{}'", s))
			)
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PowerCategory {
	High,
	Low,
	Qrp
}

impl FromStr for PowerCategory {
	type Err = CabrilloErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"HIGH" => Ok(PowerCategory::High),
			"LOW"  => Ok(PowerCategory::Low),
			"QRP"  => Ok(PowerCategory::Qrp),
			_ => Err(
				CabrilloErrorKind::ParseError(format!("Invalid value '{}'", s))
			)
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StationCategory {
	Fixed,
	Mobile,
	Portable,
	Rover,
	RoverLimited,
	RoverUnlimited,
	Expedition,
	Hq,
	School
}

impl FromStr for StationCategory {
	type Err = CabrilloErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"FIXED"           => Ok(StationCategory::Fixed),
			"MOBILE"          => Ok(StationCategory::Mobile),
			"PORTABLE"        => Ok(StationCategory::Portable),
			"ROVER"           => Ok(StationCategory::Rover),
			"ROVER-LIMITED"   => Ok(StationCategory::RoverLimited),
			"ROVER-UNLIMITED" => Ok(StationCategory::RoverUnlimited),
			"EXPEDITION"      => Ok(StationCategory::Expedition),
			"HQ"              => Ok(StationCategory::Hq),
			"SCHOOL"          => Ok(StationCategory::School),
			_ => Err(
				CabrilloErrorKind::ParseError(format!("Invalid value '{}'", s))
			)
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TimeCategory {
	Hours6,
	Hours12,
	Hours24
}

impl FromStr for TimeCategory {
	type Err = CabrilloErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"6-HOURS"  => Ok(TimeCategory::Hours6),
			"12-HOURS" => Ok(TimeCategory::Hours12),
			"24-HOURS" => Ok(TimeCategory::Hours24),
			_ => Err(
				CabrilloErrorKind::ParseError(format!("Invalid value '{}'", s))
			)
		}
	}
}	

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TransmitterCategory {
	One,
	Two,
	Limited,
	Unlimited,
	Swl
}

impl FromStr for TransmitterCategory {
	type Err = CabrilloErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"ONE"       => Ok(TransmitterCategory::One),
			"TWO"       => Ok(TransmitterCategory::Two),
			"LIMITED"   => Ok(TransmitterCategory::Limited),
			"UNLIMITED" => Ok(TransmitterCategory::Unlimited),
			"SWL"       => Ok(TransmitterCategory::Swl),
			_ => Err(
				CabrilloErrorKind::ParseError(format!("Invalid value '{}'", s))
			)
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OverlayCategory {
	Classic,
	Rookie,
	TbWires,
	NoviceTech,
	Over50
}

impl FromStr for OverlayCategory {
	type Err = CabrilloErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"CLASSIC"     => Ok(OverlayCategory::Classic),
			"ROOKIE"      => Ok(OverlayCategory::Rookie),
			"TB-WIRES"    => Ok(OverlayCategory::TbWires),
			"NOVICE-TECH" => Ok(OverlayCategory::NoviceTech),
			"OVER-50"     => Ok(OverlayCategory::Over50),
			_ => Err(
				CabrilloErrorKind::ParseError(format!("Invalid value '{}'", s))
			)
		}
	}
}

/// A QSO is a contact made between two stations. This type holds the relevant metadata
/// for each contact in the log.
#[derive(Debug, Clone)]
pub struct Qso {
	frequency: Frequency,
	mode: Mode,
	datetime: NaiveDateTime,
	call_sent: String,
	rst_sent: Option<SignalReport>,
	exch_sent: String,
	call_recvd: String,
	rst_recvd: Option<SignalReport>,
	exch_recvd: String,
	transmitter_id: bool
}

impl Qso {
	pub fn frequency(&self) -> &Frequency {
		&self.frequency
	}

	pub fn mode(&self) -> &Mode {
		&self.mode
	}

	/// Callsign sent during QSO.
	pub fn call_sent(&self) -> &String {
		&self.call_sent
	}

	/// Signal report sent during QSO.
	pub fn rst_sent(&self) -> &Option<SignalReport> {
		&self.rst_sent
	}

	/// Exchange information sent during QSO.
	pub fn exchange_sent(&self) -> &String {
		&self.exch_sent
	}

	/// Callsign received from other station.
	pub fn call_received(&self) -> &String {
		&self.call_recvd
	}

	/// Signal report received from other station.
	pub fn rst_received(&self) -> &Option<SignalReport> {
		&self.rst_recvd
	}

	/// Exchange information received from other station.
	pub fn exchange_received(&self) -> &String {
		&self.exch_recvd
	}
}

// NOTE: actually I don't believe this spec provides a way to determine *which* of the
// operators was off duty during this Offtime.

/// This type represents a period in time where an operator in this log was 
/// no longer operating.
#[derive(Debug, Clone)]
pub struct Offtime {
	begin: NaiveDateTime,
	end: NaiveDateTime
}

impl Offtime {
	pub fn begin(&self) -> &NaiveDateTime {
		&self.begin
	}

	pub fn end(&self) -> &NaiveDateTime {
		&self.end
	}
}

#[derive(Debug, Clone)]
pub struct CabrilloLog {
	version: f32,
	callsign: Option<String>,
	contest: Option<String>,
	category_assisted: Option<bool>,
	category_band: Option<Band>,
	category_mode: Option<Mode>,
	category_operator: Option<OperatorCategory>,
	category_power: Option<PowerCategory>,
	category_station: Option<StationCategory>,
	category_time: Option<TimeCategory>,
	category_transmitter: Option<TransmitterCategory>,
	category_overlay: Option<OverlayCategory>,
	certificate: Option<bool>,
	claimed_score: Option<u32>,
	club: Option<String>,
	created_by: Option<String>,
	email: Option<String>,
	grid_locator: Option<String>,
	location: Option<String>,
	name: Option<String>,
	address: Option<String>,
	operators: Vec<String>,
	offtimes: Vec<Offtime>,
	soapbox: Option<String>,
	other_tags: HashMap<String, String>,
	entries: Vec<Qso>,
	ignored_entries: Vec<Qso>,
	debug: bool
}

impl CabrilloLog {
	pub fn new() -> Self {
		Self {
			version: 3.0,
			callsign: None,
			contest: None,
			category_assisted: None,
			category_band: None,
			category_mode: None,
			category_operator: None,
			category_power: None,
			category_station: None,
			category_time: None,
			category_transmitter: None,
			category_overlay: None,
			certificate: None,
			claimed_score: None,
			club: None,
			created_by: None,
			email: None,
			grid_locator: None,
			location: None,
			name: None,
			address: None,
			operators: vec![],
			offtimes: vec![],
			soapbox: None,
			other_tags: HashMap::new(),
			entries: vec![],
			ignored_entries: vec![],
			debug: false
		}
	}

	pub fn from_buffer(buf: &[u8]) -> CabrilloResult<Self> {
		let mut new_log = Self::new();
		let mut line_no = 0;

		for line in buf.split(|c| c == &b'\n') {
			let line = str::from_utf8(&line)
				.map_err(|err| {
					CabrilloError::new("", line_no, 
						CabrilloErrorKind::IoError(
							format!("{}", err)))
				})?;

			new_log.parse_line(line_no, &line)?;
			line_no += 1;
		}

		Ok(new_log)
	}
	
	pub fn from_reader<R: BufRead>(reader: &mut R) -> CabrilloResult<Self> {
		let mut new_log = Self::new();
		let mut line_no = 0;

		for line in reader.lines() {
			let line = line
				.map_err(|err| {
					CabrilloError::new("", line_no, 
						CabrilloErrorKind::IoError(err
							.get_ref()
							.map(|v| format!("{}", v))
							.unwrap_or("Unknown I/O error".into())))
				})?;

			new_log.parse_line(line_no, &line)?;
			line_no += 1;
		}

		Ok(new_log)
	}

	fn parse_line(&mut self, line_no: usize, line: &str) -> CabrilloResult<()> {
		if line.len() == 0 {
			return Ok(());
		}

		match cabrillo_tag(&line) {
			Ok(result) => {
				let tag = (result.1).0;
				let value = (result.1).1;
				self.parse_tag(line_no, tag, value)?;
			},
			Err(error) => {
				return Err(
					CabrilloError::new("", line_no, 
						CabrilloErrorKind::ParseError(
							format!("{}", error)))
				);
			}
		}

		Ok(())
	}

	fn parse_qso_format1(&self, line_no: usize, tag: &str, value: &str) -> CabrilloResult<Qso> {
		let qso_data = cabrillo_qso_format1(value)
			.map_err(|_| {
				CabrilloError::new(tag, line_no, 
					CabrilloErrorKind::ParseError(
						format!("Invalid value '{}' (not valid QSO format)", value)))
			})?;
		let qso_data = qso_data.1;

		// FIXME: impl FromStr for Frequency
		// FIXME: this does not validate the data and the provided frequency may not be in KHz
		let frequency: Frequency = qso_data.0.parse::<u32>()
			.map(|freq| Frequency::Khz(freq))
			.map_err(|_| {
				CabrilloError::new(tag, line_no, 
					CabrilloErrorKind::ParseError(
						format!("Invalid value '{}' (invalid frequency)", value)))
			})?;

		let mode: Mode = qso_data.1.parse()
			.map_err(|err_kind| CabrilloError::new(tag, line_no, err_kind))?;

		let timestamp: NaiveDateTime = NaiveDateTime::parse_from_str(qso_data.2, "%Y-%m-%d %H%M")
			.map_err(|_| {
				CabrilloError::new(tag, line_no, 
					CabrilloErrorKind::ParseError(
						format!("Invalid value '{}' (invalid timestamp)", value)))
			})?;

		let sent_call: String = qso_data.3.to_string();
		// attempt to extract signal report from exchange info
		let sent_rst: Option<SignalReport> = qso_data.4.parse::<SignalReport>().ok();
		let sent_exch: String = format!("{} {}", qso_data.4, qso_data.5);

		let recvd_call: String = qso_data.6.to_string();
		// attempt to extract signal report from exchange info
		let recvd_rst: Option<SignalReport> = qso_data.7.parse::<SignalReport>().ok();
		let recvd_exch = format!("{} {}", qso_data.7, qso_data.8);

		Ok(Qso {
			frequency: frequency,
			mode: mode,
			datetime: timestamp,
			call_sent: sent_call,
			rst_sent: sent_rst,
			exch_sent: sent_exch,
			call_recvd: recvd_call,
			rst_recvd: recvd_rst,
			exch_recvd: recvd_exch,
			transmitter_id: false
		})
	}

	fn parse_qso_format2(&self, line_no: usize, tag: &str, value: &str) -> CabrilloResult<Qso> {
		let qso_data = cabrillo_qso_format2(value)
			.map_err(|_| {
				CabrilloError::new(tag, line_no, 
					CabrilloErrorKind::ParseError(
						format!("Invalid value '{}' (not valid QSO format)", value)))
			})?;
		let qso_data = qso_data.1;

		let frequency: Frequency = qso_data.0.parse::<u32>()
			.map(|freq| Frequency::Khz(freq))
			.map_err(|_| {
				CabrilloError::new(tag, line_no, 
					CabrilloErrorKind::ParseError(
						format!("Invalid value '{}' (invalid frequency)", value)))
			})?;

		let mode: Mode = qso_data.1.parse()
			.map_err(|err_kind| CabrilloError::new(tag, line_no, err_kind))?;

		let timestamp: NaiveDateTime = NaiveDateTime::parse_from_str(qso_data.2, "%Y-%m-%d %H%M")
			.map_err(|_| {
				CabrilloError::new(tag, line_no, 
					CabrilloErrorKind::ParseError(
						format!("Invalid value '{}' (invalid timestamp)", value)))
			})?;

		let sent_call: String = qso_data.3.to_string();
		let sent_exch: String = qso_data.4.to_string();
		let recvd_call: String = qso_data.5.to_string();
		let recvd_exch = qso_data.6.to_string();

		Ok(Qso {
			frequency: frequency,
			mode: mode,
			datetime: timestamp,
			call_sent: sent_call,
			rst_sent: None,
			exch_sent: sent_exch,
			call_recvd: recvd_call,
			rst_recvd: None,
			exch_recvd: recvd_exch,
			transmitter_id: false
		})
	}

	fn parse_tag(&mut self, line_no: usize, tag: &str, value: &str) -> CabrilloResult<()> {
		let value = value.trim();
		let parse_error = CabrilloError::new(tag, line_no, 
			CabrilloErrorKind::ParseError(format!("Invalid value '{}'", value)));
 
		match tag {
			"START-OF-LOG" => {
				self.version = value.parse()
					.map_err(|err| CabrilloError::new(tag, line_no,
						CabrilloErrorKind::ParseError(
							format!("{}", err)))
					)?;
			},
			"CALLSIGN" => {
				self.callsign = Some(value.to_string());
			},
			"CONTEST" => {
				self.contest = Some(value.to_string());
			},
			"CATEGORY-ASSISTED" => {
				self.category_assisted = match value {
					"ASSISTED" => Some(true),
					"NON-ASSISTED" => Some(false),
					_ => return Err(parse_error.clone())
				};
			},
			"CATEGORY-BAND" => {
				let band: Band = value
					.parse()
					.map_err(|err_kind| CabrilloError::new(tag, line_no, err_kind))?;
				self.category_band = Some(band);
			},
			"CATEGORY-MODE" => {
				let mode: Mode = value
					.parse()
					.map_err(|err_kind| CabrilloError::new(tag, line_no, err_kind))?;
				self.category_mode = Some(mode);
			},
			"CATEGORY-OPERATOR" => {
				let op: OperatorCategory = value
					.parse()
					.map_err(|err_kind| CabrilloError::new(tag, line_no, err_kind))?;
				self.category_operator = Some(op);
			},
			"CATEGORY-POWER" => {
				let power: PowerCategory = value
					.parse()
					.map_err(|err_kind| CabrilloError::new(tag, line_no, err_kind))?;
				self.category_power = Some(power);
			},
			"CATEGORY-STATION" => {
				let category: StationCategory = value
					.parse()
					.map_err(|err_kind| CabrilloError::new(tag, line_no, err_kind))?;
				self.category_station = Some(category);
			},
			"CATEGORY-TIME" => {
				let time: TimeCategory = value
					.parse()
					.map_err(|err_kind| CabrilloError::new(tag, line_no, err_kind))?;
				self.category_time = Some(time)
			},
			"CATEGORY-TRANSMITTER" => {
				let category: TransmitterCategory = value
					.parse()
					.map_err(|err_kind| CabrilloError::new(tag, line_no, err_kind))?;
				self.category_transmitter = Some(category);
			},
			"CATEGORY-OVERLAY" => {
				let overlay: OverlayCategory = value
					.parse()
					.map_err(|err_kind| CabrilloError::new(tag, line_no, err_kind))?;
				self.category_overlay = Some(overlay);
			},
			"CERTIFICATE" => {
				self.certificate = match value {
					"YES" => Some(true),
					"NO" => Some(false),
					_ => return Err(parse_error.clone())
				}
			},
			"CLAIMED-SCORE" => {
				self.claimed_score = value
					.parse()
					.map_err(|_| parse_error.clone())
					.ok();
			},
			"CLUB" => {
				self.club = Some(value.to_string());
			},
			"CREATED-BY" => {
				self.created_by = Some(value.to_string());
			},
			"EMAIL" => {
				cabrillo_email(value)
					.map(|email| Some((email.1).to_string()))
					.map_err(|_| {
						CabrilloError::new(tag, line_no, 
							CabrilloErrorKind::ParseError(
								format!("Invalid value '{}' (not a valid email address)", value)))
					})?;
			},
			"GRID-LOCATOR" => {
				cabrillo_grid_locator(value)
					.map(|grid| Some((grid.1).to_string()))
					.map_err(|_| {
						CabrilloError::new(tag, line_no, 
							CabrilloErrorKind::ParseError(
								format!("Invalid value '{}' (not a valid grid locator)", value)))
					})?;
			},
			"LOCATION" => {
				self.location = Some(value.to_string());
			}
			"NAME" => {
				self.name = Some(value.to_string());
			},
			"ADDRESS" | "ADDRESS-CITY" | "ADDRESS-STATE-PROVINCE" | "ADDRESS-POSTALCODE" | "ADDRESS-COUNTRY" => {
				if let Some(ref mut address) = self.address {
					address.push('\n');
					address.push_str(value);
				} else {
					self.address = Some(value.to_string());
				}
			},
			"OPERATORS" => {
				// Note: I completely gave up on doing this in nom because it was far far too difficult to deal with
				// the nuances surrounding separated_nonempty_list and complete/streaming
				// FIXME: validate callsigns
				value
					.split(|c| c == ',' || c == ' ')
					.filter(|call| call.len() > 0)
					.for_each(|call| {
						self.operators.push(call.trim().to_string())
					});
			},
			"OFFTIME" => {
				let offtime = cabrillo_offtime(value)
					.map_err(|_| {
						CabrilloError::new(tag, line_no, 
							CabrilloErrorKind::ParseError(
								format!("Invalid value '{}' (invalid timestamp format", value)))
					})?;

				let start_time = NaiveDateTime::parse_from_str((offtime.1).0, "%Y-%m-%d %H%M")
					.map_err(|_| {
						CabrilloError::new(tag, line_no, 
							CabrilloErrorKind::ParseError(
								format!("Invalid value '{}' (invalid begin time)", value)))
					})?;

				let stop_time = NaiveDateTime::parse_from_str((offtime.1).1, "%Y-%m-%d %H%M")
					.map_err(|_| {
						CabrilloError::new(tag, line_no, 
							CabrilloErrorKind::ParseError(
								format!("Invalid value '{}' (invalid end time)", value)))
					})?;

				self.offtimes.push(Offtime {
					begin: start_time,
					end: stop_time
				});
			},
			"SOAPBOX" => {
				if let Some(ref mut soapbox) = self.soapbox {
					soapbox.push('\n');
					soapbox.push_str(value);
				} else {
					self.soapbox = Some(value.to_string());
				}
			},
			"QSO" | "X-QSO" => {
				// first attempt to parse QSO in format1 then try format2 as a fallback
				// if all else fails, convert the error type into CabrilloError and fail
				// out of this function
				let qso = self.parse_qso_format1(line_no, tag, value)
					.or_else(|_| self.parse_qso_format2(line_no, tag, value))?;

				if tag == "QSO" {
					self.entries.push(qso);
				} else {
					self.ignored_entries.push(qso);
				}
			},
			"DEBUG" => {
				self.debug = true;
			},
			"END-OF-LOG" => {},
			_ => {
				self.other_tags.insert(tag.to_string(), value.to_string());
			}
		}

		Ok(())
	}

	/// Version of the Cabrillo format this log uses.
	pub fn version(&self) -> f32 {
		self.version
	}

	/// The callsign used during the contest.
	pub fn callsign(&self) -> &Option<String> {
		&self.callsign
	}

	/// The name of the contest this log is for.
	pub fn contest(&self) -> &Option<String> {
		&self.contest
	}

	pub fn category_assisted(&self) -> &Option<bool> {
		&self.category_assisted
	}

	pub fn category_band(&self) -> &Option<Band> {
		&self.category_band
	}

	pub fn category_mode(&self) -> &Option<Mode> {
		&self.category_mode
	}

	pub fn category_operator(&self) -> &Option<OperatorCategory> {
		&self.category_operator
	}

	pub fn category_power(&self) -> &Option<PowerCategory> {
		&self.category_power
	}

	pub fn category_station(&self) -> &Option<StationCategory> {
		&self.category_station
	}

	pub fn category_time(&self) -> &Option<TimeCategory> {
		&self.category_time
	}

	pub fn category_transmitter(&self) -> &Option<TransmitterCategory> {
		&self.category_transmitter
	}

	pub fn category_overlay(&self) -> &Option<OverlayCategory> {
		&self.category_overlay
	}

	/// Indicates if the operator wishes to receive, if eligible, a paper 
	/// certificate sent via postal mail by the contest sponsor.
	pub fn certificate(&self) -> &Option<bool> {
		&self.certificate
	}

	/// The claimed score of the log submission.
	pub fn claimed_score(&self) -> &Option<u32> {
		&self.claimed_score
	}

	/// The name of the club submitting this log.
	pub fn club(&self) -> &Option<String> {
		&self.club
	}

	/// The name of the software used to create this log.
	pub fn created_by(&self) -> &Option<String> {
		&self.created_by
	}

	/// Contact email address for the entrant.
	pub fn email(&self) -> &Option<String> {
		&self.email
	}

	/// The Maidenhead Grid Square where the station was operating from.
	pub fn grid_locator(&self) -> &Option<String> {
		&self.grid_locator
	}

	/// The name of the location where the station was operating from.
	pub fn location(&self) -> &Option<String> {
		&self.location
	}

	/// Name of the contact person submitting this log.
	pub fn name(&self) -> &Option<String> {
		&self.name
	}

	/// Mailing address for this log.
	pub fn address(&self) -> &Option<String> {
		&self.address
	}

	/// List of operators callsigns in this log. The host station may be indicated
	/// by an '@' character in front of their callsign.
	pub fn operators(&self) -> &Vec<String> {
		&self.operators
	}

	/// List of time ranges where breaks were taken.
	pub fn offtimes(&self) -> &Vec<Offtime> {
		&self.offtimes
	}

	/// All of the comments from this log.
	pub fn soapbox(&self) -> &Option<String> {
		&self.soapbox
	}

	/// A key-value map of all unrecognized tags in this log. Some contests use custom
	/// or non-standard tags. Those tags will be found in this map.
	pub fn other_tags(&self) -> &HashMap<String, String> {
		&self.other_tags
	}

	/// List of all QSO entries in this log.
	pub fn entries(&self) -> &Vec<Qso> {
		&self.entries
	}

	/// List of all *ignored* QSO entries as indicated by 'X-QSO'.
	pub fn ignored_entries(&self) -> &Vec<Qso> {
		&self.ignored_entries
	}

	/// Whether or not debug mode is enabled for this log.
	pub fn debug(&self) -> bool {
		self.debug
	}
}

#[cfg(test)]
mod tests {
	use std::fs::{self, File};
	use std::io::BufReader;
	use crate::*;
	
	#[test]
	fn new_from_buffer() {
		// test files
		[
			"test_data/neqp.txt",
			"test_data/rdxc.txt",
			"test_data/cqww.txt",
			"test_data/cqww_vhf.txt",
			"test_data/cqwpx.txt",
			"test_data/cqwpx_rtty.txt",
			"test_data/ncj_naqp.txt"
		]
			.iter()
			.for_each(|path| {
				let buf = fs::read(&path).unwrap();
				let log = CabrilloLog::from_buffer(&buf);

				if let Err(ref error) = log {
					eprintln!("FAILED '{}' = {:#?}", path, error);
				}

				assert!(log.is_ok());
			});
	}

	#[test]
	fn new_from_reader() {
		let data_file = File::open("test_data/afs_phone.txt").unwrap();
		let mut data_reader = BufReader::new(data_file);
		let _log = CabrilloLog::from_reader(&mut data_reader).unwrap();
	}

	#[test]
	fn frequency() {
		assert_eq!(Frequency::Khz(146520).as_mhz(), Some(146.520));
		assert_eq!(Frequency::Khz(2400000).as_ghz(), Some(2.4));
		assert_eq!(Frequency::Light.as_mhz(), None);
		assert_eq!(Frequency::Light.as_ghz(), None);
	}

	#[test]
	fn parse_tag() {
		let result = cabrillo_tag("VERSION: 2.0\n");
		assert_eq!(result, Ok(("\n", ("VERSION", "2.0"))));
	}

	#[test]
	fn parse_email() {
		let result = cabrillo_email("name@test.com");
		assert_eq!(result, Ok(("", "name@test.com")));

		let result = cabrillo_email("fasdklfj@lakjsdf");
		assert!(result.is_err());

		let result = cabrillo_email("893u4f9834.com");
		assert!(result.is_err());
	}

	#[test]
	fn parse_grid_locator() {
		let result = cabrillo_grid_locator("FN20ib");
		assert_eq!(result, Ok(("", "FN20ib")));

		let result = cabrillo_grid_locator("ar34id");
		assert_eq!(result, Ok(("", "ar34id")));

		let result = cabrillo_grid_locator("Az99xx");
		assert!(result.is_err());

		let result = cabrillo_grid_locator("asdf");
		assert!(result.is_err());

		let result = cabrillo_grid_locator("FN20id00xx");
		assert!(result.is_err());
	}

	#[test]
	fn parse_callsign() {
		["K3AH", "VK2ABCD/M", "2E0ABC", "@W1AW/p"]
			.iter()
			.for_each(|call| {
				let _ = cabrillo_callsign(call).unwrap();
			});
	}

	#[test]
	fn parse_signal_report() {
		let rst: SignalReport = "599".parse().unwrap();
		assert_eq!(rst, SignalReport(5, 9, 9));

		let rst: SignalReport = "34".parse().unwrap();
		assert_eq!(rst, SignalReport(3, 4, 0));

		["7", "00", "000", "asd", "999"]
			.iter()
			.for_each(|signal| {
				let rst: Result<SignalReport, CabrilloErrorKind> = signal.parse();
				assert!(rst.is_err());
			});
	}
}
