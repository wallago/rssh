use strum::EnumIter;

#[derive(Clone, Copy, PartialEq, Debug, EnumIter, strum::Display)]
pub enum Filter {
    All,
    Unread,
    Starred,
}
