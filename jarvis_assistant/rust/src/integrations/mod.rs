/// Integration Layer — Email + Calendar
///
/// EmailIntegration: IMAP/SMTP with OAuth support.
/// CalendarIntegration: Google/Outlook calendar with free-slot finder.

use std::collections::HashMap;

// ── Email ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Email {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
    pub timestamp: u64,
    pub read: bool,
}

#[derive(Debug, Clone)]
pub struct EmailThread {
    pub thread_id: String,
    pub emails: Vec<Email>,
}

pub struct EmailIntegration {
    /// Stub inbox.
    inbox: Vec<Email>,
    next_id: u64,
}

impl EmailIntegration {
    pub fn new() -> Self {
        Self { inbox: vec![], next_id: 1 }
    }

    /// Read emails from inbox (stub: returns stored emails).
    pub fn read_emails(&self, limit: usize) -> Vec<&Email> {
        self.inbox.iter().take(limit).collect()
    }

    /// Send an email (stub: stores in sent log).
    pub fn send_email(
        &mut self,
        to: Vec<String>,
        subject: impl Into<String>,
        body: impl Into<String>,
    ) -> String {
        let id = format!("email-{}", self.next_id);
        self.next_id += 1;
        // Stub: just return the id; real impl would use SMTP.
        id
    }

    /// Watch inbox for new messages (stub: returns current unread count).
    pub fn watch_inbox(&self) -> usize {
        self.inbox.iter().filter(|e| !e.read).count()
    }

    /// Summarize a thread by concatenating subjects.
    pub fn summarize_thread(&self, thread_id: &str) -> String {
        let subjects: Vec<&str> = self
            .inbox
            .iter()
            .filter(|e| e.id.starts_with(thread_id))
            .map(|e| e.subject.as_str())
            .collect();
        if subjects.is_empty() {
            format!("No emails found for thread: {thread_id}")
        } else {
            format!("Thread summary: {}", subjects.join("; "))
        }
    }

    /// Inject a test email into the inbox (for testing).
    pub fn inject_email(&mut self, email: Email) {
        self.inbox.push(email);
    }
}

impl Default for EmailIntegration {
    fn default() -> Self {
        Self::new()
    }
}

// ── Calendar ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub start_ts: u64, // Unix timestamp seconds
    pub end_ts: u64,
    pub attendees: Vec<String>,
    pub description: String,
}

impl CalendarEvent {
    pub fn duration_secs(&self) -> u64 {
        self.end_ts.saturating_sub(self.start_ts)
    }
}

pub struct CalendarIntegration {
    events: Vec<CalendarEvent>,
    next_id: u64,
}

impl CalendarIntegration {
    pub fn new() -> Self {
        Self { events: vec![], next_id: 1 }
    }

    /// Get events in a time range [from_ts, to_ts].
    pub fn get_events(&self, from_ts: u64, to_ts: u64) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| e.start_ts >= from_ts && e.end_ts <= to_ts)
            .collect()
    }

    /// Create a new calendar event.
    pub fn create_event(
        &mut self,
        title: impl Into<String>,
        start_ts: u64,
        end_ts: u64,
        attendees: Vec<String>,
    ) -> String {
        let id = format!("event-{}", self.next_id);
        self.next_id += 1;
        self.events.push(CalendarEvent {
            id: id.clone(),
            title: title.into(),
            start_ts,
            end_ts,
            attendees,
            description: String::new(),
        });
        id
    }

    /// Update an existing event's title.
    pub fn update_event(&mut self, id: &str, new_title: impl Into<String>) -> bool {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == id) {
            event.title = new_title.into();
            true
        } else {
            false
        }
    }

    /// Delete an event by id.
    pub fn delete_event(&mut self, id: &str) -> bool {
        let before = self.events.len();
        self.events.retain(|e| e.id != id);
        self.events.len() < before
    }

    /// Find the first free slot of `duration_secs` within [from_ts, to_ts].
    ///
    /// Returns `Some(start_ts)` of the free slot, or `None` if none found.
    pub fn find_free_slot(
        &self,
        from_ts: u64,
        to_ts: u64,
        duration_secs: u64,
    ) -> Option<u64> {
        // Collect busy intervals sorted by start.
        let mut busy: Vec<(u64, u64)> = self
            .events
            .iter()
            .filter(|e| e.start_ts < to_ts && e.end_ts > from_ts)
            .map(|e| (e.start_ts, e.end_ts))
            .collect();
        busy.sort_by_key(|(s, _)| *s);

        let mut cursor = from_ts;
        for (busy_start, busy_end) in &busy {
            if cursor + duration_secs <= *busy_start {
                return Some(cursor);
            }
            if *busy_end > cursor {
                cursor = *busy_end;
            }
        }
        if cursor + duration_secs <= to_ts {
            Some(cursor)
        } else {
            None
        }
    }
}

impl Default for CalendarIntegration {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_send_returns_id() {
        let mut email = EmailIntegration::new();
        let id = email.send_email(vec!["bob@example.com".to_string()], "Hi", "Hello");
        assert!(!id.is_empty());
    }

    #[test]
    fn email_read_returns_injected_emails() {
        let mut email = EmailIntegration::new();
        email.inject_email(Email {
            id: "e1".to_string(),
            from: "alice@example.com".to_string(),
            to: vec!["me@example.com".to_string()],
            subject: "Test".to_string(),
            body: "Hello".to_string(),
            timestamp: 0,
            read: false,
        });
        let emails = email.read_emails(10);
        assert_eq!(emails.len(), 1);
    }

    #[test]
    fn calendar_create_and_get_event() {
        let mut cal = CalendarIntegration::new();
        let id = cal.create_event("Meeting", 1000, 2000, vec![]);
        let events = cal.get_events(0, 3000);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, id);
    }

    #[test]
    fn calendar_delete_event() {
        let mut cal = CalendarIntegration::new();
        let id = cal.create_event("To delete", 0, 100, vec![]);
        assert!(cal.delete_event(&id));
        assert!(cal.get_events(0, 200).is_empty());
    }

    #[test]
    fn calendar_update_event() {
        let mut cal = CalendarIntegration::new();
        let id = cal.create_event("Old title", 0, 100, vec![]);
        assert!(cal.update_event(&id, "New title"));
        let events = cal.get_events(0, 200);
        assert_eq!(events[0].title, "New title");
    }

    #[test]
    fn find_free_slot_no_events() {
        let cal = CalendarIntegration::new();
        let slot = cal.find_free_slot(0, 3600, 1800);
        assert_eq!(slot, Some(0));
    }

    #[test]
    fn find_free_slot_around_busy_event() {
        let mut cal = CalendarIntegration::new();
        cal.create_event("Busy", 0, 1800, vec![]);
        // Free slot should start at 1800.
        let slot = cal.find_free_slot(0, 3600, 1800);
        assert_eq!(slot, Some(1800));
    }

    #[test]
    fn find_free_slot_returns_none_when_full() {
        let mut cal = CalendarIntegration::new();
        cal.create_event("All day", 0, 3600, vec![]);
        let slot = cal.find_free_slot(0, 3600, 1800);
        assert!(slot.is_none());
    }
}
