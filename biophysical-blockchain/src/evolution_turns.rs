use chrono::{Datelike, Utc};
use crate::sealed::inner::Sealed;

/// Per-host, per-UTC-day evolution turn counter.
#[derive(Clone, Debug)]
pub struct DailyTurnState {
    pub date_yyyymmdd: u32,
    pub turns_used: u8,
}

impl Sealed for DailyTurnState {}

impl DailyTurnState {
    pub fn new_for_today() -> Self {
        let now = Utc::now().date_naive();
        let d = (now.year() as u32) * 10_000
            + (now.month() as u32) * 100
            + now.day() as u32;
        DailyTurnState {
            date_yyyymmdd: d,
            turns_used: 0,
        }
    }

    fn refresh_day(&mut self) {
        let now = Utc::now().date_naive();
        let d = (now.year() as u32) * 10_000
            + (now.month() as u32) * 100
            + now.day() as u32;
        if d != self.date_yyyymmdd {
            self.date_yyyymmdd = d;
            self.turns_used = 0;
        }
    }

    /// Try to consume one evolution turn.
    /// Returns true if a turn is available; false if daily cap is reached.
    pub fn try_consume_turn(&mut self, max_turns: u8) -> bool {
        self.refresh_day();
        if self.turns_used < max_turns {
            self.turns_used += 1;
            true
        } else {
            false
        }
    }
}

/// Hard ceiling, compiled into inner-ledger.
/// Governance shards may only lower this, never raise it.
pub const MAX_DAILY_TURNS_INNER: u8 = 10;
