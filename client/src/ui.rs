// UI system. HUD overlay, menus, inventory screens.
// In v0.0.1: stub module. Full UI rendering is TODO.

/// HUD state for rendering overlay elements.
pub struct HudState {
    /// Show debug info (F3).
    pub show_debug: bool,
    /// Show chat.
    pub show_chat: bool,
    /// Show inventory screen.
    pub show_inventory: bool,
    /// Show pause menu.
    pub show_pause_menu: bool,
    /// Chat input buffer.
    pub chat_input: String,
    /// Chat message history (last N messages).
    pub chat_messages: Vec<ChatEntry>,
}

pub struct ChatEntry {
    pub message: String,
    pub timestamp: std::time::Instant,
}

impl HudState {
    pub fn new() -> Self {
        Self {
            show_debug: false,
            show_chat: false,
            show_inventory: false,
            show_pause_menu: false,
            chat_input: String::new(),
            chat_messages: Vec::new(),
        }
    }

    /// Add a chat message to history.
    pub fn push_chat(&mut self, message: String) {
        self.chat_messages.push(ChatEntry {
            message,
            timestamp: std::time::Instant::now(),
        });
        // Keep last 100 messages
        if self.chat_messages.len() > 100 {
            self.chat_messages.remove(0);
        }
    }

    /// Get visible chat messages (last 10 sec or if chat is open).
    pub fn visible_chat(&self) -> Vec<&str> {
        let now = std::time::Instant::now();
        self.chat_messages.iter()
            .filter(|m| self.show_chat || now.duration_since(m.timestamp).as_secs() < 10)
            .map(|m| m.message.as_str())
            .collect()
    }
}
