#[macro_use]
extern crate lazy_static;
extern crate nom;
extern crate chrono;

use std::str;
use std::io::BufRead;
use std::fmt::{self, Display};
use std::error::Error;
use std::convert::TryFrom;
use std::collections::HashMap;
use chrono::NaiveDateTime;
use nom::{
	IResult,
	branch::alt,
	multi::{
		many1,
		many_m_n,
		fold_many1
	},
	combinator::{
		eof,
		not,
		opt,
		value,
		recognize,
		complete,
		map,
		map_res
	},
	sequence::{
		tuple,
		preceded,
		terminated,
		separated_pair
	},
	bytes::complete::{
		tag,
		take_while_m_n
	},
	character::complete::{
		digit1,
		space0,
		space1,
		alphanumeric1,
		not_line_ending,
		one_of,
		char
	}
};

macro_rules! parser_map {
	(<$type: ty> $($key: expr => $value: expr),*) => {{
		let mut map = HashMap::new();
		$( map.insert($key, $value as $type); )*
		map
	}}
}

lazy_static! {
	static ref TAGS: HashMap<&'static str, for<'a> fn(&'a str, &'a mut CabrilloLog) -> IResult<&'a str, ()>> = {
		parser_map![
			<for<'a> fn(&'a str, &'a mut CabrilloLog) -> IResult<&'a str, ()>> 
			"START-OF-LOG"         => cabrillo_log_start,
			"CALLSIGN"             => cabrillo_log_callsign,
			"CONTEST"              => cabrillo_log_contest,
			"CATEGORY-ASSISTED"    => cabrillo_log_category_assisted,
			"CATEGORY-BAND"        => cabrillo_log_category_band,
			"CATEGORY-MODE"        => cabrillo_log_category_mode,
			"CATEGORY-OPERATOR"    => cabrillo_log_category_operator,
			"CATEGORY-POWER"       => cabrillo_log_category_power,
			"CATEGORY-STATION"     => cabrillo_log_category_station,
			"CATEGORY-TIME"        => cabrillo_log_category_time,
			"CATEGORY-TRANSMITTER" => cabrillo_log_category_xmitter,
			"CATEGORY-OVERLAY"     => cabrillo_log_category_overlay,
			"CERTIFICATE"          => cabrillo_log_certificate,
			"CLAIMED-SCORE"        => cabrillo_log_claimed_score,
			"CLUB"                 => cabrillo_log_club,
			"CREATED-BY"           => cabrillo_log_created_by,
			"EMAIL"                => cabrillo_log_email,
			"GRID-LOCATOR"         => cabrillo_log_grid_locator,
			"LOCATION"             => cabrillo_log_location,
			"NAME"                 => cabrillo_log_name,
			"ADDRESS"              => cabrillo_log_addr_fragment,
			"ADDRESS-CITY"         => cabrillo_log_addr_fragment,
			"ADDRESS-STATE-PROVINCE" => cabrillo_log_addr_fragment,
			"ADDRESS-POSTALCODE"   => cabrillo_log_addr_fragment,
			"ADDRESS-COUNTRY"      => cabrillo_log_addr_fragment,
			"OPERATORS"            => cabrillo_log_operators,
			"OFFTIME"              => cabrillo_log_offtime,
			"SOAPBOX"              => cabrillo_log_soapbox,
			"X-QSO"                => cabrillo_ignore_qso,
			"QSO"                  => cabrillo_log_qso,
			"DEBUG"                => cabrillo_log_debug,
			"END-OF-LOG"           => cabrillo_log_end
		]
	};
}

fn cabrillo_tag(input: &str) -> IResult<&str, (&str, &str)> {
	alt((
		complete(
			separated_pair(
				recognize(many1(one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-' "))),
				tag(": "), // the spec requires a space after the colon
				not_line_ending
			)
		),
		map(tag("END-OF-LOG"), |s| (s, ""))
	))(input)
}

fn cabrillo_email_chars(input: &str) -> IResult<&str, &str> {
	recognize(
		alt((
			alphanumeric1,
			recognize(many1(one_of("_-.")))
		))
	)(input)
}

fn cabrillo_email(input: &str) -> IResult<&str, &str> {
	recognize(
		tuple((
			cabrillo_email_chars,
			tag("@"),
			cabrillo_email_chars,
			tag("."),
			take_while_m_n(2, 5, char::is_alphanumeric)
		))
	)(input)
}

fn cabrillo_grid_locator(input: &str) -> IResult<&str, &str> {
	recognize(
		tuple((
			many_m_n(2, 2, one_of("ABCDEFGHIJKLMNOPQR")),
			many_m_n(2, 2, one_of("0123456789")),
			opt(many_m_n(2, 2, one_of("abcdefghijklmnopqrstuvwxyz"))),
			opt(many_m_n(2, 2, one_of("0123456789"))),
			eof
		))
	)(input)
}

// named!(
// 	cabrillo_grid_locator<&str, &str>),
// 	re_match_static!(r"^([A-Ra-r]{2})([0-9]{2})([A-Ra-r]{2}){0,1}$")
// );

fn cabrillo_callsign(input: &str) -> IResult<&str, &str> {
	recognize(
		tuple((
			opt(tag("@")),
			take_while_m_n(3, 8, char::is_alphanumeric),
			opt(
				preceded(
					tag("/"),
					take_while_m_n(1, 8, char::is_alphanumeric)
				)
			)
		))
	)(input)
}

fn cabrillo_datetime(input: &str) -> IResult<&str, NaiveDateTime> {
	map_res(
		recognize(
			tuple((
				take_while_m_n(4, 4, |c: char| c.is_digit(10)),
				tag("-"),
				take_while_m_n(2, 2, |c: char| c.is_digit(10)),
				tag("-"),
				take_while_m_n(2, 2, |c: char| c.is_digit(10)),
				tag(" "),
				take_while_m_n(4, 4, |c: char| c.is_digit(10))
			))
		),
		|date_str: &str| NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H%M")
	)(input)
}

fn cabrillo_mode(input: &str) -> IResult<&str, Mode> {
	alt((
		value(Mode::Cw     , tag("CW")),
		value(Mode::Fm     , tag("FM")),
		value(Mode::Phone  , alt((tag("PH"), tag("SSB")))),
		value(Mode::Rtty   , alt((tag("RY"), tag("RTTY")))),
		value(Mode::Digital, alt((tag("DG"), tag("DIGI")))),
		value(Mode::Mixed  , tag("MIXED"))
	))(input)
}

fn cabrillo_offtime(input: &str) -> IResult<&str, Offtime> {
	map(
		separated_pair(
			cabrillo_datetime,
			char(' '),
			cabrillo_datetime
		),
		|time_pair: (NaiveDateTime, NaiveDateTime)| {
			Offtime {
				begin: time_pair.0,
				end: time_pair.1
			}
		}
	)(input)
}

fn cabrillo_frequency(input: &str) -> IResult<&str, Frequency> {
	map(
		map_res(
			digit1,
			|digits: &str| digits.parse::<u32>()
		),
		Frequency::Khz
	)(input)
}

/*fn cabrillo_signal_report(input: &str) -> IResult<&str, SignalReport> {
	map(
		tuple((
			map_opt(
				one_of("12345")),
				|readability: char| readability.to_digit(10)
			)),
			map_opt(
				one_of("123456789")),
				|strength: char| strength.to_digit(10)
			)),
			alt((
				map_opt(
					one_of("123456789")),
					|tone: char| tone.to_digit(10)
				)),
				value(0, eof)
			))
		))),
		|rst: (u32, u32, u32)| SignalReport(rst.0 as u8, rst.1 as u8, rst.2 as u8)
	)(input)
}*/

fn cabrillo_operators(input: &str) -> IResult<&str, Vec<String>> {
	fold_many1(
		terminated(
			cabrillo_callsign,
			opt(
				terminated(
					tag(","),
					space0
				)
			)
		),
		Vec::new,
		|mut callsigns: Vec<_>, item| {
			callsigns.push(item.to_string());
			callsigns
		}
	)(input)
}

fn cabrillo_qso(input: &str) -> IResult<&str, Qso> {
	map(
		preceded(
			space0,
			tuple((
				terminated(
					cabrillo_frequency, // Frequency
					space1
				),
				terminated(
					cabrillo_mode,      // Mode
					space1
				),
				terminated(
					cabrillo_datetime,  // QSO timestamp
					space1,
				),
				terminated(
					cabrillo_callsign,  // Sent call
					space1
				),
				terminated(             // Sent exchange
					alt((
						map(
							separated_pair(
								alphanumeric1,
								space1,
								recognize(
									tuple((
										not(cabrillo_callsign),
										alphanumeric1
									))
								)
							),
							|pair: (&str, &str)| format!("{} {}", pair.0, pair.1)
						),
						map(alphanumeric1, |i: &str| i.to_string())
					)),
					space1
				),
				terminated(
					cabrillo_callsign,  // Rcvd call
					space1
				),
				terminated(             // Recvd exchange
					alt((
						map(
							separated_pair(
								alphanumeric1,
								space1,
								recognize(
									tuple((
										not(cabrillo_callsign),
										alphanumeric1
									))
								)
							),
							|pair: (&str, &str)| format!("{} {}", pair.0, pair.1)
						),
						map(alphanumeric1, |i: &str| i.to_string())
					)),
					space0
				)
			)),
		),
		|data: (Frequency, Mode, NaiveDateTime, &str, String, &str, String)| {
			Qso {
				frequency: data.0,
				mode: data.1,
				datetime: data.2,
				call_sent: data.3.to_string(),
				exch_sent: data.4,
				call_recvd: data.5.to_string(),
				exch_recvd: data.6,
				transmitter_id: false
			}
		}
	)(input)
}

fn cabrillo_log_start<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		alt((tag("2.0"), tag("3.0"))),
		|version: &str| {
			log.version = version.parse::<f32>().unwrap()
		}
	)(input)
}

fn cabrillo_log_callsign<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		cabrillo_callsign,
		|call: &str| log.callsign = Some(call.to_string())
	)(input)
}

fn cabrillo_log_contest<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	log.contest = Some(input.trim().to_string());
	Ok(("", ()))
}

fn cabrillo_log_category_assisted<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		alt((
			value(Some(true) , tag("ASSISTED")),
			value(Some(false), tag("NON-ASSISTED"))
		)),
		|yesno: Option<bool>| log.category_assisted = yesno
	)(input)
}

fn cabrillo_log_category_band<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		alt((
			alt((
				value(Band::All      , tag("ALL")),
				value(Band::Band160M , tag("160M")),
				value(Band::Band80M  , tag("80M")),
				value(Band::Band40M  , tag("40M")),
				value(Band::Band20M  , tag("20M")),
				value(Band::Band15M  , tag("15M")),
				value(Band::Band10M  , tag("10M")),
				value(Band::Band6M   , tag("6M")),
				value(Band::Band4M   , tag("4M")),
				value(Band::Band2M   , tag("2M")),
				value(Band::Band222  , tag("222")),
				value(Band::Band432  , tag("432")),
				value(Band::Band902  , tag("902")),
				value(Band::Band1_2G , tag("1.2G")),
				value(Band::Band2_3G , tag("2.3G")),
				value(Band::Band3_4G , tag("3.4G")),
				value(Band::Band5_7G , tag("5.7G")),
				value(Band::Band10G  , tag("10G")),
				value(Band::Band24G  , tag("24G")),
				value(Band::Band47G  , tag("47G")),
			)),
			alt((
				value(Band::Band75G  , tag("75G")),
				value(Band::Band123G , tag("123G")),
				value(Band::Band134G , tag("134G")),
				value(Band::Band241G , tag("241G")),
				value(Band::Light    , tag("LIGHT")),
				value(Band::Vhf3Band , tag("VHF-3-BAND")),
				value(Band::VhfFmOnly, tag("VHF-FM-ONLY"))
			))
		)),
		|band: Band| log.category_band = Some(band)
	)(input)
}

fn cabrillo_log_category_mode<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		cabrillo_mode,
		|mode: Mode| log.category_mode = Some(mode)
	)(input)
}

fn cabrillo_log_category_operator<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		alt((
			value(OperatorCategory::SingleOp, tag("SINGLE-OP")),
			value(OperatorCategory::MultiOp , tag("MULTI-OP")),
			value(OperatorCategory::CheckLog, tag("CHECKLOG"))
		)),
		|op: OperatorCategory| log.category_operator = Some(op)
	)(input)
}

fn cabrillo_log_category_power<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		alt((
			value(PowerCategory::High, tag("HIGH")),
			value(PowerCategory::Low , tag("LOW")),
			value(PowerCategory::Qrp , tag("QRP"))
		)),
		|power: PowerCategory| log.category_power = Some(power)
	)(input)
}

fn cabrillo_log_category_station<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		alt((
			value(StationCategory::Fixed         , tag("FIXED")),
			value(StationCategory::Mobile        , tag("MOBILE")),
			value(StationCategory::Portable      , tag("PORTABLE")),
			value(StationCategory::Rover         , tag("ROVER")),
			value(StationCategory::RoverLimited  , tag("ROVER-LIMITED")),
			value(StationCategory::RoverUnlimited, tag("ROVER-UNLIMITED")),
			value(StationCategory::Expedition    , tag("EXPEDITION")),
			value(StationCategory::Hq            , tag("HQ")),
			value(StationCategory::School        , tag("SCHOOL"))
		)),
		|st: StationCategory| log.category_station = Some(st)
	)(input)
}

fn cabrillo_log_category_time<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		alt((
			value(TimeCategory::Hours6 , tag("6-HOURS")),
			value(TimeCategory::Hours12, tag("12-HOURS")),
			value(TimeCategory::Hours24, tag("24-HOURS"))
		)),
		|time: TimeCategory| log.category_time = Some(time)
	)(input)
}

fn cabrillo_log_category_xmitter<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		alt((
			value(TransmitterCategory::One      , tag("ONE")),
			value(TransmitterCategory::Two      , tag("TWO")),
			value(TransmitterCategory::Limited  , tag("LIMITED")),
			value(TransmitterCategory::Unlimited, tag("UNLIMITED")),
			value(TransmitterCategory::Swl      , tag("SWL"))
		)),
		|xmitter: TransmitterCategory| log.category_transmitter = Some(xmitter)
	)(input)
}

fn cabrillo_log_category_overlay<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		alt((
			value(OverlayCategory::Classic   , tag("CLASSIC")),
			value(OverlayCategory::Rookie    , tag("ROOKIE")),
			value(OverlayCategory::TbWires   , tag("TB-WIRES")),
			value(OverlayCategory::NoviceTech, tag("NOVICE-TECH")),
			value(OverlayCategory::Over50    , tag("OVER-50"))
		)),
		|overlay: OverlayCategory| log.category_overlay = Some(overlay)
	)(input)
}

fn cabrillo_log_certificate<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		alt((
			value(Some(true) , tag("YES")),
			value(Some(false), tag("NO"))
		)),
		|yesno: Option<bool>| log.certificate = yesno
	)(input)
}

fn cabrillo_log_claimed_score<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		map_res(
			digit1,
			|score: &str| score.parse::<u32>()
		),
		|score: u32| log.claimed_score = Some(score)
	)(input)
}

fn cabrillo_log_club<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	log.club = Some(input.trim().to_string());
	Ok(("", ()))
}

fn cabrillo_log_created_by<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	log.created_by = Some(input.trim().to_string());
	Ok(("", ()))
}

fn cabrillo_log_email<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		cabrillo_email,
		|email: &str| log.email = Some(email.to_string())
	)(input)
}

fn cabrillo_log_grid_locator<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		cabrillo_grid_locator,
		|grid_square: &str| log.grid_locator = Some(grid_square.to_string())
	)(input)
}

fn cabrillo_log_location<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	log.location = Some(input.trim().to_string());
	Ok(("", ()))
}

fn cabrillo_log_name<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	log.name = Some(input.trim().to_string());
	Ok(("", ()))
}

fn cabrillo_log_addr_fragment<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	let value = input.trim();

	if let Some(ref mut address) = log.address {
		address.push('\n');
		address.push_str(value);
	} else {
		log.address = Some(value.to_string());
	}
	
	Ok(("", ()))
}

fn cabrillo_log_operators<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		cabrillo_operators,
		|ops: Vec<String>| log.operators.extend(ops)
	)(input)
}

fn cabrillo_log_offtime<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		cabrillo_offtime,
		|offtime: Offtime| log.offtimes.push(offtime)
	)(input)
}

fn cabrillo_log_soapbox<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	let value = input.trim();

	if let Some(ref mut soapbox) = log.soapbox {
		soapbox.push('\n');
		soapbox.push_str(value);
	} else {
		log.soapbox = Some(value.to_string());
	}
	
	Ok(("", ()))
}

fn cabrillo_log_qso<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		cabrillo_qso,
		|qso: Qso| log.entries.push(qso)
	)(input)
}

fn cabrillo_ignore_qso<'a>(input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	map(
		cabrillo_qso,
		|qso: Qso| log.ignored_entries.push(qso)
	)(input)
}

fn cabrillo_log_debug<'a>(_input: &'a str, log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	log.debug = true;
	Ok(("", ()))
}

fn cabrillo_log_end<'a>(_input: &'a str, _log: &'a mut CabrilloLog) -> IResult<&'a str, ()> {
	Ok(("", ()))
}

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
			line,
			kind
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

impl TryFrom<Frequency> for Band {
	type Error = CabrilloErrorKind;

	fn try_from(other: Frequency) -> Result<Self, Self::Error> {
		match other {
			Frequency::Khz(freq) => {
				match freq {
					_ if (1800..=2000).contains(&freq) => Ok(Band::Band160M),
					_ if (3500..=4000).contains(&freq) => Ok(Band::Band80M),
					_ if (7000..=7300).contains(&freq) => Ok(Band::Band40M),
					_ if (14000..=14350).contains(&freq) => Ok(Band::Band20M),
					_ if (21000..=21450).contains(&freq) => Ok(Band::Band15M),
					_ if (28000..=29700).contains(&freq) => Ok(Band::Band10M),
					_ if (50000..=54000).contains(&freq) => Ok(Band::Band6M),
					_ if (70000..=70500).contains(&freq) => Ok(Band::Band4M),
					_ if (144000..=148000).contains(&freq) => Ok(Band::Band2M),
					_ if (219000..=225000).contains(&freq) => Ok(Band::Band222),
					_ if (420000..=450000).contains(&freq) => Ok(Band::Band432),
					_ if (902000..=928000).contains(&freq) => Ok(Band::Band902),
					_ if (1240000..=1300000).contains(&freq) => Ok(Band::Band1_2G),
					_ if (2390000..=2450000).contains(&freq) => Ok(Band::Band2_3G),
					_ if (3300000..=3500000).contains(&freq) => Ok(Band::Band3_4G),
					_ if (5650000..=5925000).contains(&freq) => Ok(Band::Band5_7G),
					_ if (10000000..=10500000).contains(&freq) => Ok(Band::Band10G),
					_ if (24000000..=24250000).contains(&freq) => Ok(Band::Band24G),
					_ if (47000000..=47200000).contains(&freq) => Ok(Band::Band47G),
					_ if (76000000..=81000000).contains(&freq) => Ok(Band::Band75G),
					_ if (122250000..=123000000).contains(&freq) => Ok(Band::Band123G),
					_ if (134000000..=141000000).contains(&freq) => Ok(Band::Band134G),
					_ if (241000000..=250000000).contains(&freq) => Ok(Band::Band241G),
					_ if freq >= 300000000 => Ok(Band::Light),
					_ => Err(
						CabrilloErrorKind::ParseError(format!("The value '{}' does not fall within a valid amateur band", other.to_string()))
					)
				}
			},
			Frequency::Light => Ok(Band::Light)
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

/// A tuple type representing the 3 parts of a signal report (readability, strength, and tone). If the tone
/// will always be zero if it is not provided.
/*#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SignalReport(u8, u8, u8);*/

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperatorCategory {
	SingleOp,
	MultiOp,
	CheckLog
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PowerCategory {
	High,
	Low,
	Qrp
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TimeCategory {
	Hours6,
	Hours12,
	Hours24
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TransmitterCategory {
	One,
	Two,
	Limited,
	Unlimited,
	Swl
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OverlayCategory {
	Classic,
	Rookie,
	TbWires,
	NoviceTech,
	Over50
}

/// A QSO is a contact made between two stations. This type holds the relevant metadata
/// for each contact in the log.
#[derive(Debug, Clone)]
pub struct Qso {
	frequency: Frequency,
	mode: Mode,
	datetime: NaiveDateTime,
	call_sent: String,
	exch_sent: String,
	call_recvd: String,
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

	pub fn datetime(&self) -> &NaiveDateTime {
		&self.datetime
	}

	/// Callsign sent during QSO.
	pub fn call_sent(&self) -> &String {
		&self.call_sent
	}

	/// Exchange information sent during QSO.
	pub fn exchange_sent(&self) -> &String {
		&self.exch_sent
	}

	/// Callsign received from other station.
	pub fn call_received(&self) -> &String {
		&self.call_recvd
	}

	/// Exchange information received from other station.
	pub fn exchange_received(&self) -> &String {
		&self.exch_recvd
	}

	pub fn transmitter_id(&self) -> bool {
		self.transmitter_id
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

#[derive(Debug, Default, Clone)]
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
			..Default::default()
		}
	}

	pub fn from_buffer(buf: &[u8]) -> CabrilloResult<Self> {
		let mut new_log = Self::new();

		for (line_no, line) in buf.split(|c| c == &b'\n').enumerate() {
			let line = str::from_utf8(line)
				.map_err(|err| {
					CabrilloError::new("", line_no, 
						CabrilloErrorKind::IoError(
							format!("{}", err)))
				})?;

			new_log.parse_line(line_no, line)?;
		}

		Ok(new_log)
	}
	
	pub fn from_reader<R: BufRead>(reader: &mut R) -> CabrilloResult<Self> {
		let mut new_log = Self::new();

		for (line_no, line) in reader.lines().enumerate() {
			let line = line
				.map_err(|err| {
					CabrilloError::new("", line_no, 
						CabrilloErrorKind::IoError(err
							.get_ref()
							.map(|v| format!("{}", v))
							.unwrap_or_else(|| "Unknown I/O error".into())))
				})?;

			new_log.parse_line(line_no, &line)?;
		}

		Ok(new_log)
	}

	fn parse_line(&mut self, line_no: usize, line: &str) -> CabrilloResult<()> {
		if line.is_empty() {
			return Ok(());
		}

		match cabrillo_tag(line) {
			Ok((_, (tag, value))) => {
				self.parse_tag(line_no, tag, value)?;
			},
			Err(error) => {
				return Err(
					CabrilloError::new("", line_no, 
						CabrilloErrorKind::ParseError(error.to_string()))
				);
			}
		}

		Ok(())
	}

	fn parse_tag(&mut self, line_no: usize, tag: &str, value: &str) -> CabrilloResult<()> {
 		match TAGS.get(tag) {
 			Some(parser) => {
 				parser(value, self)
 					.map_err(|error| {
 						CabrilloError::new(
 							tag, 
 							line_no, 
							CabrilloErrorKind::ParseError(
								error.to_string()
							)
						)
					})?;
 			},
 			None => {
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

		let result = cabrillo_grid_locator("AR34id11");
		assert_eq!(result, Ok(("", "AR34id11")));

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

	/*#[test]
	fn parse_signal_report() {
		let rst = cabrillo_signal_report("599");
		assert_eq!(rst, Ok(("", SignalReport(5, 9, 9))));

		let rst = cabrillo_signal_report("34");
		assert_eq!(rst, Ok(("", SignalReport(3, 4, 0))));

		["7", "00", "000", "asd", "999"]
			.iter()
			.for_each(|signal| {
				let rst = cabrillo_signal_report(signal);
				assert!(rst.is_err());
			});
	}*/
}
