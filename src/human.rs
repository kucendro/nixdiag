use bytesize::ByteSize;
use jiff::Timestamp;
use num_format::{Locale, ToFormattedString};
use std::fmt::{self, Display, Formatter};

pub const DAY: i64 = 86_400;

pub struct Bytes(pub u64);
pub struct Count(pub usize);
pub struct Date(pub i64);

impl Display for Bytes {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        ByteSize(self.0).display().iec().fmt(f)
    }
}

impl Display for Count {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.0.to_formatted_string(&Locale::en))
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match Timestamp::from_second(self.0) {
            Ok(t) => t.strftime("%F").fmt(f),
            Err(_) => self.0.fmt(f),
        }
    }
}
