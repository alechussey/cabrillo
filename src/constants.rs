const TAGS: &'static [&'static str] = &[
	"START-OF-LOG",
	"END-OF-LOG",
	"CALLSIGN",
	"CONTEST",
	"CATEGORY-ASSISTED",
	"CATEGORY-MODE",
	"CATEGORY-OPERATOR",
	"CATEGORY-POWER",
	"CATEGORY-STATION",
	"CATEGORY-TIME",
	"CATEGORY-TRANSMITTER",
	"CATEGORY-OVERLAY",
	"CERTIFICATE",
	"CLAIMED-SCORE",
	"CLUB",
	"CREATED-BY",
	"EMAIL",
	"GRID-LOCATOR",
	"LOCATION",
	"NAME",
	"ADDRESS",
	"ADDRESS-CITY",
	"ADDRESS-STATE-PROVINCE",
	"ADDRESS-POSTALCODE",
	"ADDRESS-COUNTRY",
	"OPERATORS",
	"OFFTIME",
	"SOAPBOX",
	"QSO",
	"X-QSO",
	"DEBUG"
];

const CATEGORY_ASSISTED_VALUES: &'static [&'static str] = &[
	"ASSISTED", "NOT-ASSISTED"
];

const CATEGORY_BAND_VALUES: &'static [&'static str] = &[
	"ALL", "160M", "80M", "40M", "20M", "15M", "10M", "6M", "2M", "222",
	"432", "902", "1.2G", "2.3G", "3.4G", "5.7G", "10G", "24G", "47G",
	"75G", "123G", "134G", "241G", "Light", "VHF-3-BAND", "VHF-FM-ONLY"
];

const CATEOGORY_MODE_VALUES: &'static [&'static str] = &[
	"CW", "DIGI", "FM", "RTTY", "SSB", "MIXED"
];

const CATEGORY_OPERATOR_VALUES: &'static [&'static str] = &[
	"SIGNLE-OP", "MULTI-OP", "CHECKLOG"
];

const CATEGORY_POWER_VALUES: &'static [&'static str] = &[
	"HIGH", "LOW", "QRP"
];

const CATEGORY_TIME_VALUES: &'static [&'static str] = &[
	"6-HOURS", "12-HOURS", "24-HOURS"
];

const CATEGORY_OVERLAY_VALUES: &'static [&'static str] = &[
	"CLASSIC", "ROOKIE", "TB-WIRES", "NOVICE-TECH", "OVER-50"
];
