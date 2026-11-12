use crate::{application::ApplicationError, domain::DomainError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub has_more: bool,
}

impl<T> Page<T> {
    pub fn from_items(mut items: Vec<T>, limit: u64) -> Self {
        let has_more = u64::try_from(items.len()).is_ok_and(|count| count > limit);
        if has_more {
            items.truncate(usize::try_from(limit).unwrap_or(usize::MAX));
        }
        Self { items, has_more }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageRequest {
    limit: u64,
    offset: u64,
}

impl PageRequest {
    pub fn new(limit: u64, offset: u64) -> Result<Self, ApplicationError> {
        if !(1..=100).contains(&limit) || offset > 1_000_000 {
            return Err(DomainError::Validation(
                "Page size must be from 1 to 100 and offset cannot exceed 1000000.",
            )
            .into());
        }
        Ok(Self { limit, offset })
    }

    pub fn limit(self) -> u64 {
        self.limit
    }

    pub fn offset(self) -> u64 {
        self.offset
    }

    pub fn fetch_limit(self) -> u64 {
        self.limit + 1
    }
}
