use std::fmt::Display;

use chrono::{Duration, Utc, DateTime};

use crate::traits::C32;

pub struct PrettyDuration(pub Duration);

impl Display for PrettyDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0.num_milliseconds())
    }
}

pub fn duration(start_time: &Option<DateTime<Utc>>, end_time: &Option<DateTime<Utc>>) -> Option<Duration>{
    match start_time {
        Some(val) => end_time.map(|end_t| end_t.signed_duration_since(val)),
        None => None,
    }
}

pub fn is_zero(c: &C32) -> bool {
    c.re == 0.0 && c.im == 0.0 
}