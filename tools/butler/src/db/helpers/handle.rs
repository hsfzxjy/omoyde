use super::*;
use std::collections::btree_map::{Entry, OccupiedEntry};

pub type Occupied<'a, T> = OccupiedEntry<'a, <T as Table>::PrimaryKey, <T as Table>::Record>;
type TableHandleInner<'a, T> = Option<Entry<'a, <T as Table>::PrimaryKey, <T as Table>::Record>>;

pub struct TableHandle<'a, T: Table>(pub(super) TableHandleInner<'a, T>);

impl<'a, T: Table> From<TableHandleInner<'a, T>> for TableHandle<'a, T> {
    fn from(x: TableHandleInner<'a, T>) -> Self {
        Self(x)
    }
}

impl<'a, T: Table> TableHandle<'a, T> {
    pub(super) fn into_mut(self) -> Option<TableRecordMut<'a, T>> {
        self.into_occupied().map(OccupiedEntry::into_mut)
    }
    pub(super) fn into_occupied(self) -> Option<Occupied<'a, T>> {
        match self.0 {
            Some(Entry::Occupied(x)) => Some(x),
            _ => None,
        }
    }
}

impl<'a, T: Table> From<Option<TableHandle<'a, T>>> for TableHandle<'a, T> {
    fn from(v: Option<TableHandle<'a, T>>) -> Self {
        match v {
            None => Self(None),
            Some(x) => x,
        }
    }
}

pub type TableRecordMut<'a, T> = &'a mut <T as Table>::Record;
