use std::fmt::Display;

use humantime::format_rfc3339;

use crate::ts_to_systemtime;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Feed {
        entry_id: String,
        time: i64,
        payload: String,
    },
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Feed { entry_id, time, .. } => {
                write!(
                    f,
                    "Feed event: {} at {}",
                    entry_id,
                    format_rfc3339(ts_to_systemtime(*time as u64))
                )
            }
        }
    }
}
