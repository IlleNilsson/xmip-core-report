#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use xmip_core::{JourneyId, MessageId};

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
