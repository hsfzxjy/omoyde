use crate::prelude::*;

impl<'b, 'a: 'b> TableEntryTrait<'b, 'a, MountPointTable> for TableEntry<'b, 'a, MountPointTable> {
    type Patch = MountPointRecordPatch<'b, 'a>;
}

impl<'b, 'a: 'b> TableAccessMut<'a, MountPointTable> {
    pub fn insert_or_update(&'b mut self, path: CanonicalizedPath, alias: Option<String>) {
        self.entry(path.clone())
            .or_insert_with(|| MountPointRecord::new(path, None))
            .set_alias_with(|x| {
                if alias.is_some() {
                    *x = alias
                }
            })
            .commit();
    }
}

pub fn mpt_access<'a>() -> TableAccess<'a, MountPointTable> {
    MOUNTPOINT_TABLE.lock().unwrap().access()
}

pub fn mpt_access_mut<'a>() -> TableAccessMut<'a, MountPointTable> {
    MOUNTPOINT_TABLE.lock().unwrap().access_mut()
}
