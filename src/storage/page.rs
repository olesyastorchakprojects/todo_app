use crate::handlers::PaginationParams;

pub trait HasId<Id> {
    fn id(&self) -> Id;
}

#[derive(Debug, Copy, Clone)]
pub struct Pagination<Id> {
    pub after: Option<Id>,
    pub limit: usize,
}

impl<Id> From<PaginationParams<Id>> for Pagination<Id> {
    fn from(p: PaginationParams<Id>) -> Self {
        match p {
            PaginationParams::FirstPage { limit } => Pagination { after: None, limit },
            PaginationParams::NextPage { after, limit } => Pagination {
                after: Some(after),
                limit,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Page<T: HasId<Id>, Id> {
    pub items: Vec<T>,
    pub next_cursor: Option<Id>,
    pub limit: usize,
}

impl<T: HasId<Id>, Id> From<&Pagination<Id>> for Page<T, Id> {
    fn from(value: &Pagination<Id>) -> Self {
        Self {
            items: Vec::with_capacity(value.limit),
            next_cursor: None,
            limit: value.limit,
        }
    }
}

impl<T: HasId<Id>, Id> Page<T, Id> {
    pub fn complete_with(&mut self, item: T) -> bool
    where
        T: HasId<Id>,
    {
        if self.items.len() == self.limit {
            self.next_cursor = self.items.last().map(|t: &T| t.id());
            true
        } else {
            self.items.push(item);
            false
        }
    }
}
