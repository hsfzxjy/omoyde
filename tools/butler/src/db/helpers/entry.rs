use super::*;
use std::collections::btree_map::Entry;

pub struct TableEntry<'b, 'a: 'b, T: Table> {
    pub(super) handle: TableHandle<'a, T>,
    pub(in crate::db) ptr: &'b TableRef<'a, T, &'a mut T>,
}

impl<'b, 'a: 'b, T: Table> std::fmt::Debug for TableEntry<'b, 'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TableEntry")
    }
}

pub trait TableEntryTrait<'b, 'a: 'b, T: Table> {
    type Patch: TableRecordPatch<'b, 'a, Table = T>;
}

pub(super) type Patch<'b, 'a, T> = <TableEntry<'b, 'a, T> as TableEntryTrait<'b, 'a, T>>::Patch;

impl<'b, 'a: 'b, T: Table> TableEntry<'b, 'a, T> {
    pub(super) fn with_key<K>(k: K, ptr: &'b TableRef<'a, T, &'a mut T>) -> Self
    where
        K: TableKey<T>,
    {
        let handle = k.query_in(unsafe { ptr.as_mut() });
        Self { ptr, handle }
    }
    pub fn remove(self) -> Option<<T as Table>::Record> {
        let ptr = unsafe { self.ptr.as_mut() };
        self.handle.into_occupied().map(|x| ptr.remove(x))
    }
}

impl<'b, 'a: 'b, T: Table> TableEntry<'b, 'a, T>
where
    Self: TableEntryTrait<'b, 'a, T>,
{
    pub fn or_insert_with< F>(self, f: F) -> Patch<'b, 'a, T>
    where
        F: FnOnce() -> <T as Table>::Record,
    {
        let this = match self.modify() {
            Ok(x) => return x,
            Err(x) => x,
        };
        let rec = f();
        let rec = unsafe { this.ptr.as_mut() }.insert(rec);
        <Patch<'b, 'a, T,>>::new(rec, this.ptr)
    }
    pub fn modify(self) -> std::result::Result<Patch<'b, 'a, T>, Self> {
        match self.handle.0 {
            Some(Entry::Occupied(x)) => Ok(<Patch<'b, 'a, T>>::new(x.into_mut(), self.ptr)),
            _ => Err(self),
        }
    }
}
