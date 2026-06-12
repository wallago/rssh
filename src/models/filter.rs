use strum::EnumIter;

/// Filter allow us to select specific articles
#[derive(Clone, Copy, PartialEq, Debug, EnumIter, strum::Display)]
pub enum Filter {
    /// Take them all
    All,
    /// Only unreaded ones
    Unread,
    /// Only marked ones
    Starred,
}
