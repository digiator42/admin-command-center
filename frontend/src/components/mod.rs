//! Presentational + stateful components for the ACC console.
//!
//! Each is annotated with `#[component]`. Shared state is never threaded as
//! props — components pull the single `AppState` out of context with
//! `use_context`. Styling is Tailwind utilities; every class string is a full
//! literal so the Tailwind scanner sees it.

pub mod control_deck;
pub mod login_view;
pub mod metrics_tiles;
pub mod services_table;
pub mod toast_stack;
pub mod top_bar;

pub use control_deck::*;
pub use login_view::*;
pub use metrics_tiles::*;
pub use services_table::*;
pub use toast_stack::*;
pub use top_bar::*;
