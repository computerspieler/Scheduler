use chrono::{DateTime, Datelike, Duration, Months, TimeZone, Timelike, Utc};

macro_rules! check_char {
    ($time: ident [$n: literal], $c: literal) => {
        if $time.chars().nth($n)? != $c {
            return None;
        }
    };
}

pub fn get_start_timestamp_from_string(time: &str) -> Option<DateTime<Utc>> {
    /* The format follows the ISO 8601 specifications, but support blank
     * values, with asterisks, which will be replaced by instant's value.
     */
    macro_rules! get_value {
        ($elt: expr, $default: expr) => {{
            let elt = &$elt;

            let mut is_only_asterisk = true;
            let mut is_only_digit = true;

            for c in elt.chars() {
                is_only_asterisk &= (c == '*');
                is_only_digit &= (c.is_ascii_digit());
            }

            if is_only_asterisk == is_only_digit {
                return None;
            } else if is_only_asterisk {
                $default
            } else {
                match elt.parse() {
                Ok(x) => x,
                Err(_) => return None
                }
            }
        }};
    }

    let now = chrono::Utc::now();
    
    let year = get_value!(time[0 .. 4], now.year());
    check_char!(time[4], '-');
    let month = get_value!(time[5 .. 7], now.month());
    check_char!(time[7], '-');
    let day = get_value!(time[8 .. 10], now.day());
    check_char!(time[10], 'T');
    let hour = get_value!(time[11 .. 13], now.hour());
    check_char!(time[13], ':');
    let min = get_value!(time[14 .. 16], now.minute());
    check_char!(time[16], ':');
    let sec = get_value!(time[17 .. 19], now.second());

    let date = Utc.with_ymd_and_hms(year, month, day, hour, min, sec).unwrap();

    let tz_shift = {
        let c = time.chars().nth(19)?;
        match c {
        'Z' if time.len() == 20 => Duration::zero(),
        '-' | '+' => {
            let tz = &time[20 ..];
            let (h, m) =
                match tz.find(':') {
                Some(idx) if tz.len() == 5  =>
                    (&tz[0 .. idx], &tz[idx+1 .. idx+3]),
                None if tz.len() == 4 =>
                    (&tz[0 .. 2], &tz[2 .. 4]),
                _ => return None
                }
            ;
            let (h, m): (i64, i64) =
                match (h.parse(), m.parse()) {
                (Ok(h), Ok(m)) => (h, m),
                _ => return None
                };
            (Duration::hours(h) + Duration::minutes(m)) *
            if c == '+' { -1 } else { 1 }
        },

        _ => return None
        }
    };

    Some(date + tz_shift)
}

#[derive(Debug)]
pub struct YmdHmsDuration {
    year: u32,
    month: u32,
    day: i64,
    hour: i64,
    min: i64,
    sec: i64,
}

impl YmdHmsDuration {
    pub fn add(&self, other: DateTime<Utc>) -> DateTime<Utc> {
        other +
            Months::new(12 * self.year + self.month) +
            Duration::seconds(self.sec) +
            Duration::minutes(self.min) +
            Duration::hours(self.hour) +
            Duration::days(self.day)
    }
}

impl ToString for YmdHmsDuration {
    fn to_string(&self) -> std::string::String {
        format!( "{}-{}-{} {}:{}:{}",
            self.year, self.month, self.day,
            self.hour, self.min,   self.sec
        )
    }
}

pub fn get_period_from_string(time: &str) -> Option<YmdHmsDuration> {
    if time.len() != 19 {
        return None;
    }

    let year = time[0 .. 4].parse().ok()?;
    check_char!(time[4], '-');
    let month = time[5 .. 7].parse().ok()?;
    check_char!(time[7], '-');
    let day = time[8 .. 10].parse().ok()?;
    check_char!(time[10], ' ');
    let hour = time[11 .. 13].parse().ok()?;
    check_char!(time[13], ':');
    let min = time[14 .. 16].parse().ok()?;
    check_char!(time[16], ':');
    let sec = time[17 .. 19].parse().ok()?;

    Some(YmdHmsDuration {
        year: year,
        month: month,
        day: day,
        hour: hour,
        min: min,
        sec: sec
    }
    )
}