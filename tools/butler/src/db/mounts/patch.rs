use crate::prelude::*;

pub struct MountPointRecordPatch<'b, 'a: 'b> {
    rec: TableRecordMut<'a, MountPointTable>,
    _ptr: &'b TableRefMut<'a, MountPointTable>,
}

impl<'b, 'a: 'b> Drop for MountPointRecordPatch<'b, 'a> {
    fn drop(&mut self) {}
}

impl<'b, 'a: 'b> TableRecordPatch<'b, 'a> for MountPointRecordPatch<'b, 'a> {
    type Table = MountPointTable;
    fn new(rec: TableRecordMut<'a, Self::Table>, ptr: &'b TableRefMut<'a, Self::Table>) -> Self {
        Self { rec, _ptr: ptr }
    }
}

#[allow(dead_code)]
impl<'b, 'a: 'b> MountPointRecordPatch<'b, 'a> {
    pub fn set_alias(self, alias: Option<String>) -> Self {
        self.rec.alias = alias;
        self
    }
    pub fn set_alias_with<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut Option<String>),
    {
        f(&mut self.rec.alias);
        self
    }
}
