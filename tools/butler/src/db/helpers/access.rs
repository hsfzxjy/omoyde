use super::*;
use crate::prelude::*;
use crate::util::serde::TableIO;
use std::collections::btree_map::Values;

pub struct TableAccess<'a, T>(pub(in crate::db) TableRef<'a, T, &'a T>);

impl<'b, 'a: 'b, T> TableAccess<'a, T>
where
    T: Table,
{
    pub fn query<K, Q>(&'b self, q: Q) -> Option<&'a <T as Table>::Record>
    where
        K: TableKey<T>,
        Q: Borrow<K>,
    {
        q.borrow()
            .query_in(unsafe { self.0.as_mut() })
            .into_mut()
            .map(|x| &*x)
    }

    pub fn records(&'b self) -> Values<'a, <T as Table>::PrimaryKey, <T as Table>::Record> {
        unsafe { self.0.as_mut() }.treemap_mut().values()
    }
}

impl<'b, 'a: 'b, T> TableAccess<'a, T>
where
    T: Table + serde::de::DeserializeOwned + Serialize,
{
    pub fn finalize<P: AsRef<Path>>(&'b self, p: P) -> Result<()> {
        unsafe { self.0.as_mut() }.save_to_path(p)
    }
}

pub trait TableAccessExt<'a, T>
where
    Self: Deref<Target = T> + 'a,
{
    fn access(self) -> TableAccess<'a, T>;
}

impl<'a, T> TableAccessExt<'a, T> for std::sync::MutexGuard<'a, T> {
    fn access(self) -> TableAccess<'a, T> {
        TableAccess(TableRef::Guard(self))
    }
}

impl<'a, T> TableAccessExt<'a, T> for &'a T {
    fn access(self) -> TableAccess<'a, T> {
        TableAccess(TableRef::Ptr(
            unsafe { std::mem::transmute(self) },
            PhantomData,
        ))
    }
}

impl<'b, 'a: 'b, T> TableAccessMut<'a, T>
where
    T: Table + serde::de::DeserializeOwned + Serialize,
{
    pub fn initialize<P: AsRef<Path>>(&'b self, p: P) -> Result<()> {
        let table = unsafe { self.0.as_mut() };
        table.load_from_path(p)?;
        let treemap = unsafe { self.0.as_mut() }.treemap_mut();
        table.index_mut().build(treemap);
        Ok(())
    }
}

pub struct TableAccessMut<'a, T>(pub(in crate::db) TableRef<'a, T, &'a mut T>);

impl<'b, 'a: 'b, T, K: 'a> TableAccessMut<'a, T>
where
    T: Table<PrimaryKey = K>,
    K: Ord + Clone + std::hash::Hash,
{
    pub fn retain<F, P>(&'b mut self, mut f: F)
    where
        TableEntry<'b, 'a, T>: TableEntryTrait<'b, 'a, T, Patch = P>,
        P: TableRecordPatch<'b, 'a, Table = T>,
        F: FnMut(&K, &mut P) -> bool,
    {
        let mut to_remove = HashSet::new();
        let map = unsafe { self.0.as_mut() }.treemap_mut();
        for (key, value) in map.iter_mut() {
            let mut patch = Patch::<'_, '_, T>::new(value, &self.0);
            if !f(&key, &mut patch) {
                to_remove.insert(key.clone());
            }
            patch.commit();
        }

        if to_remove.is_empty() {
            return;
        }

        let table_index = unsafe { self.0.as_mut() }.index_mut();
        let map = unsafe { self.0.as_mut() }.treemap_mut();
        map.retain(|k, rec| {
            if to_remove.contains(k) {
                table_index.remove(rec);
                false
            } else {
                true
            }
        });

        unsafe { self.0.as_mut() }.modified_flag().set();
    }
}

impl<'b, 'a: 'b, T> TableAccessMut<'a, T>
where
    T: Table,
{
    pub fn entry<K>(&'b mut self, k: K) -> TableEntry<'b, 'a, T>
    where
        K: TableKey<T>,
    {
        TableEntry::with_key(k, &self.0)
    }
    pub fn insert(&'b mut self, rec: <T as Table>::Record) -> &'b <T as Table>::Record {
        let table = unsafe { self.0.as_mut() };
        table.modified_flag().set();
        table.insert(rec)
    }
}

impl<'a, T> Deref for TableAccessMut<'a, T> {
    type Target = TableAccess<'a, T>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}

pub trait TableAccessMutExt<'a, T>
where
    Self: DerefMut<Target = T> + 'a,
{
    fn access_mut(self) -> TableAccessMut<'a, T>;
}

impl<'a, T> TableAccessMutExt<'a, T> for std::sync::MutexGuard<'a, T> {
    fn access_mut(self) -> TableAccessMut<'a, T> {
        TableAccessMut(TableRef::Guard(self))
    }
}

impl<'a, T> TableAccessMutExt<'a, T> for &'a mut T {
    fn access_mut(self) -> TableAccessMut<'a, T> {
        TableAccessMut(TableRef::Ptr(self, PhantomData))
    }
}
