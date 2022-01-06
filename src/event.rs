use std::fmt::Display;

use humantime::format_rfc3339;

use crate::ts_to_systemtime;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Feed {
        entry_id: String,
        time: i64,
        content: Option<String>,
        title: Option<String>,
        link: Option<String>,
    },
    CratesIo {
        name: String,
        vers: String,
        links: Option<String>,
        yanked: bool,
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
            Event::CratesIo { name, .. } => write!(f, "Crates.io event: {} ", name),
        }
    }
}
