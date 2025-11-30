pub mod channel;
pub mod duplicate_filter;

pub use channel::{AuthorType, ChannelManager, ChannelType, Message};
pub use duplicate_filter::{DuplicateCheckResult, DuplicateFilter, DuplicateFilterConfig};
