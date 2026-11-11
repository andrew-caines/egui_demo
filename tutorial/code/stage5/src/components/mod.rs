//! Reusable UI building blocks. Each component lives in its own file and only the
//! types re-exported here are part of the module's public surface.

mod add_bar;
mod edit_dialog;
mod filter_panel;
mod import_dialog;
mod priority_badge;
mod progress_ring;
mod settings_window;
mod todo_menu;
mod todo_row;
mod todo_table;

pub use add_bar::AddBar;
pub use edit_dialog::{DialogOutcome, EditDialog};
pub use filter_panel::FilterPanel;
pub use import_dialog::{ImportDialog, ImportOutcome};
pub use priority_badge::PriorityBadge;
pub use progress_ring::ProgressRing;
pub use settings_window::SettingsWindow;
pub use todo_menu::todo_context_menu;
pub use todo_row::TodoRow;
pub use todo_table::TodoTable;
