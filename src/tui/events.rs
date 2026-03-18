//! Event handling for the TUI using crossterm.
//!
//! Spawns a background thread that polls for terminal events and emits
//! `Event::Key`, `Event::Resize`, or `Event::Tick` on a channel.

use crossterm::event::{self, KeyEvent};
use std::sync::mpsc;
use std::time::Duration;

/// Events produced by the event handler.
#[derive(Debug)]
pub enum Event {
    /// A key was pressed.
    Key(KeyEvent),
    /// The terminal was resized.
    Resize(u16, u16),
    /// A tick elapsed (for periodic redraws).
    Tick,
}

/// Polls crossterm events on a background thread and sends them through a channel.
pub struct EventHandler {
    rx: mpsc::Receiver<Event>,
    // Keep the handle alive so the thread doesn't get detached unexpectedly.
    _tx: mpsc::Sender<Event>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate in milliseconds.
    pub fn new(tick_rate_ms: u64) -> Self {
        let (tx, rx) = mpsc::channel();
        let event_tx = tx.clone();
        let tick_duration = Duration::from_millis(tick_rate_ms);

        std::thread::spawn(move || {
            loop {
                // Poll with the tick duration as timeout
                if event::poll(tick_duration).unwrap_or(false) {
                    match event::read() {
                        Ok(event::Event::Key(key)) => {
                            // Only send key press events (ignore release/repeat on some platforms)
                            if key.kind == crossterm::event::KeyEventKind::Press {
                                if event_tx.send(Event::Key(key)).is_err() {
                                    return;
                                }
                            }
                        }
                        Ok(event::Event::Resize(w, h)) => {
                            if event_tx.send(Event::Resize(w, h)).is_err() {
                                return;
                            }
                        }
                        Ok(_) => {
                            // Ignore mouse events, focus events, etc.
                        }
                        Err(_) => {
                            return;
                        }
                    }
                } else {
                    // Timeout: emit a tick
                    if event_tx.send(Event::Tick).is_err() {
                        return;
                    }
                }
            }
        });

        Self { rx, _tx: tx }
    }

    /// Block until the next event is available.
    pub fn next(&self) -> anyhow::Result<Event> {
        self.rx
            .recv()
            .map_err(|e| anyhow::anyhow!("Event channel closed: {e}"))
    }
}
