use chrono::prelude::*;

pub fn current_time(tz: bool) -> DateTime<FixedOffset> {
    if tz {
        Local::now().with_timezone(&Local::now().offset().fix())
    } else {
        Utc::now().with_timezone(&Utc.fix())
    }
}