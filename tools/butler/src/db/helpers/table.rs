use super::handle::TableHandle;
use super::Occupied;
use std::collections::btree_map::BTreeMap;
use std::marker::PhantomData;
use std::ops::Deref;

pub trait Table {
    type PrimaryKey: Ord;
    type Record;

    fn insert(&mut self, rec: Self::Record) -> &mut Self::Record;
    fn remove(&mut self, slot: Occupied<'_, Self>) -> Self::Record;
    fn remove_index(&mut self, rec: &Self::Record);
    fn treemap(&mut self) -> &mut BTreeMap<Self::PrimaryKey, Self::Record>;
    fn after_init(&mut self);
}

pub trait TableKey<T: Table> {
    fn query_in<'a, 'b>(&'a self, table: &'b mut T) -> TableHandle<'b, T>;
    fn query_ref_in<'a, 'b>(&'a self, table: &'b mut T) -> Option<&'b <T as Table>::Record> {
        self.query_in(table).into_mut().map(|x| &*x)
    }
}

pub enum TableRef<'a, T, M: 'a> {
    Ptr(*mut T, PhantomData<M>),
    Guard(std::sync::MutexGuard<'a, T>),
}

impl<'b, 'a: 'b, M: 'a, T> TableRef<'a, T, M> {
    #[inline(always)]
    #[allow(mutable_transmutes)]
    pub(in crate::db) unsafe fn as_mut(&'b self) -> &'a mut T {
        use TableRef::*;
        match self {
            Ptr(x, _) => &mut **x,
            Guard(x) => std::mem::transmute(x.deref()),
        }
    }
}

pub type TableRefMut<'a, T> = TableRef<'a, T, &'a mut T>;
