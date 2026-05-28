use crate::models::category::{self, Category};

#[derive(Clone, PartialEq, Debug)]
pub struct Feed {
    pub id: String,
    pub label: String,
    pub category: Category,
    pub error_count: i64,
}
