#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use xcore::{JourneyId, MessageId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportRecord {
    pub record_type: String,
    pub journey_id: JourneyId,
    pub message_id: Option<MessageId>,
    pub timestamp_unix_nanos: i128,
    pub fields: BTreeMap<String, String>,
}

pub trait ReportDataSink: Send + Sync {
    fn write(&self, record: ReportRecord);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct Kept(Mutex<Vec<ReportRecord>>);

    impl ReportDataSink for Kept {
        fn write(&self, record: ReportRecord) {
            if let Ok(mut kept) = self.0.lock() {
                kept.push(record);
            }
        }
    }

    #[test]
    fn a_record_reaches_the_sink_with_its_fields() {
        let kept = Kept(Mutex::new(Vec::new()));
        let record = ReportRecord {
            record_type: "delivery".to_string(),
            journey_id: JourneyId::new(1),
            message_id: Some(MessageId::new(2)),
            timestamp_unix_nanos: 3,
            fields: [("bytes".to_string(), "512".to_string())]
                .into_iter()
                .collect(),
        };
        kept.write(record.clone());
        assert_eq!(kept.0.lock().expect("kept").as_slice(), &[record]);
    }
}
