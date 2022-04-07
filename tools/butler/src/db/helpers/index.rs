use super::*;
use std::collections::BTreeMap;

pub trait TableIndex {
    type Table: Table;

    fn remove(&mut self, rec: &<Self::Table as Table>::Record);
    fn insert(&mut self, rec: &<Self::Table as Table>::Record);
    fn build(
        &mut self,
        treemap: &BTreeMap<<Self::Table as Table>::PrimaryKey, <Self::Table as Table>::Record>,
    ) {
        for rec in treemap.values() {
            self.insert(rec)
        }
    }
}
