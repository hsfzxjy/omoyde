use super::handle::*;
use super::table::*;

#[allow(drop_bounds)]
pub trait TableRecordPatch<'b, 'a>: Drop + Sized {
    type Table: Table;
    fn new(rec: TableRecordMut<'a, Self::Table>, ptr: &'b TableRefMut<'a, Self::Table>) -> Self;
    fn commit(self) {
        drop(self);
    }
}
