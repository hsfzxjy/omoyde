use crate::db::*;
use crate::prelude::*;

impl TableKey<MountPointTable> for Uuid {
    fn query_in<'a, 'b>(
        &'a self,
        table: &'b mut MountPointTable,
    ) -> TableHandle<'b, MountPointTable> {
        table
            .index
            .uuid2path
            .get(self.borrow())
            .cloned()
            .map(|path| path.query_in(table))
            .into()
    }
}

impl TableKey<MountPointTable> for CanonicalizedPath {
    fn query_in<'a, 'b>(
        &'a self,
        table: &'b mut MountPointTable,
    ) -> TableHandle<'b, MountPointTable> {
        Some(table.path2rec.entry(self.clone())).into()
    }
}
