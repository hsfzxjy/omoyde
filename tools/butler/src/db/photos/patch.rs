use crate::prelude::*;
use paste::paste;
use std::borrow::Cow;
use std::ptr::NonNull;

#[derive(Clone, Debug)]
pub struct Diff<'a, T: Clone> {
    old: Cow<'a, T>,
    new: Option<Cow<'a, T>>,
}

impl<'a, T: Clone> From<T> for Diff<'a, T> {
    fn from(v: T) -> Self {
        Self {
            old: Cow::Owned(v),
            new: None,
        }
    }
}

impl<'a, T: Clone> From<&'a T> for Diff<'a, T> {
    fn from(v: &'a T) -> Self {
        Self {
            old: Cow::Borrowed(v),
            new: None,
        }
    }
}

impl<'b, 'a: 'b, 'c: 'a, T: Clone> Diff<'a, T> {
    fn write(&self, slot: &mut T) {
        self.new.as_ref().map(|v| *slot = v.clone().into_owned());
    }
    fn get_mut(&'b mut self) -> &'b mut T {
        if self.new.is_none() {
            self.new = Some(self.old.clone());
        }
        self.new.as_mut().unwrap().to_mut()
    }
    pub fn set(&mut self, v: T) {
        *self.get_mut() = v;
    }
    fn current(&self) -> &T {
        self.new.as_ref().unwrap_or(&self.old)
    }
    fn to_owned(&'b self) -> Diff<'c, T> {
        Diff {
            old: Cow::Owned(self.old.clone().into_owned()),
            new: self
                .new
                .as_ref()
                .map(|x| Cow::Owned(x.clone().into_owned())),
        }
    }
}

impl<'a, T: Eq + Clone> Diff<'a, T> {
    fn changed(&self) -> bool {
        if matches!(self.old, Cow::Borrowed(_)) && matches!(self.new, None | Some(Cow::Borrowed(_)))
        {
            return false;
        }
        self.new
            .as_ref()
            .map(|new| new.ne(&self.old))
            .unwrap_or(false)
    }
    fn run_if_changed<F>(&self, f: F)
    where
        F: FnOnce(&T, &T),
    {
        if self.changed() {
            f(&self.old, &self.new.as_ref().unwrap())
        }
    }
}

macro_rules! fields {
    ($action: ident; $args: tt) => {
        fields!{@iter $action, $args, [
            (metadata; PhotoMetadata),
            (file_hash; FileHash),
            (selected; bool),
            (status; PhotoRecordStatus),
            (commit_time; Option<DateTime<Utc>>)
        ]}
    };
    (@iter define_struct, (), [ $(( $N: ident; $T: ty )),+ ]) => {
        #[derive(Debug)]
        pub struct PhotoRecordDiff<'a> {
            pub pid: PID,
            is_missing: bool,
            $(
                pub $N: Diff<'a, $T>,
            )+
        }
    };
    (@iter to_owned, ($self: ident), [ $(( $N: ident; $T: ty )),+ ]) => {
        PhotoRecordDiff {
            pid: $self.pid,
            is_missing: $self.is_missing,
            $( $N: $self.$N.to_owned(), )+
        }
    };
    (@iter new, ($arg: ident), [ $(( $N: ident; $T: ty )),+ ]) => {
        Self {
            pid: $arg.pid,
            is_missing: false,
            $( $N: (&$arg.$N).into(), )+
        }
    };
    (@iter $action: ident, $args: tt, [ $(( $N: ident; $T: ty )),+ ]) => {
        $(
            fields!(@call $action, $args, $N, $T);
        )+
    };
    (@call write, ($self: ident, $arg: ident, $changed: ident), $N: ident, $T: ty) => {
        if $self.$N.changed() {
            $self.$N.old.to_mut();
            $self.$N.write(&mut $arg.$N);
            $changed = true;
        }
    };
    (@call accessor, (), $N: ident, $T: ty) => {
        paste!{
            pub fn $N(&self) -> &$T {
                self.rec_diff.$N.current()
            }
            pub fn [<set_ $N>](&mut self, v: $T) {
                *self.rec_diff.$N.get_mut() = v;
            }
            pub fn [<$N _mut>](&mut self) -> &mut $T {
                self.rec_diff.$N.get_mut()
            }
            pub fn [<with_ $N>](mut self, v: $T) -> Self {
                self.[<set_ $N>](v);
                self
            }
            pub fn [<set_ $N _with>]<F>(mut self, f: F) -> Self
            where
                F: FnOnce(&mut $T)
            {
                f(self.rec_diff.$N.get_mut());
                self
            }
        }
    }
}

fields!(define_struct; ());

impl<'a> PhotoRecordDiff<'a> {
    fn write<'b, 'c>(&'b mut self, rec: &'c mut PhotoRecord) -> (&'c PhotoRecord, bool) {
        let mut changed = false;
        fields!(write; (self, rec, changed));
        (rec, changed)
    }
    pub fn new(rec: &'a PhotoRecord) -> Self {
        fields!(new; (rec))
    }
    fn is_dirty(&self) -> bool {
        self.file_hash.changed() || self.metadata.changed()
    }
    fn to_owned<'c: 'a>(&self) -> PhotoRecordDiff<'c> {
        fields!(to_owned; (self))
    }
}

pub struct PhotoRecordPatch<'b, 'a: 'b> {
    commit_at_drop: bool,
    rec_diff: PhotoRecordDiff<'a>,
    rec: NonNull<PhotoRecord>,
    ptr: &'b TableRefMut<'a, PhotoTable>,
}

#[allow(dead_code)]
impl<'b, 'a: 'b> PhotoRecordPatch<'b, 'a> {
    fields!(accessor; ());
    pub(super) fn with_diff<'c: 'a>(mut self, diff: PhotoRecordDiff<'c>) -> Self {
        self.rec_diff = diff;
        self
    }
    pub fn mark_missing(&mut self) {
        self.rec_diff.is_missing = true
    }
    pub fn into_diff<'c>(mut self) -> PhotoRecordDiff<'c> {
        self.commit_at_drop = false;
        let ret = self.rec_diff.to_owned();
        drop(self);
        ret
    }
}

impl<'b, 'a: 'b> Drop for PhotoRecordPatch<'b, 'a> {
    fn drop(&mut self) {
        if !self.commit_at_drop {
            return;
        }
        let ptr = unsafe { self.ptr.as_mut() };

        if *self.selected() && self.rec_diff.is_missing {
            self.status_mut().handle_local_missing();
        } else {
            let is_dirty = self.rec_diff.is_dirty();
            self.status_mut().handle_dirty_mark(is_dirty);
        }
        {
            let status = &self.rec_diff.status;
            if status.changed() && *status.current() == Committed {
                self.set_commit_time(Some(Utc::now()))
            }
        }

        let (rec, changed) = self.rec_diff.write(unsafe { self.rec.as_mut() });

        if changed {
            unsafe { self.ptr.as_mut() }.modified_flag().set();
        }

        self.rec_diff.selected.run_if_changed(|_o, _n| {
            ptr.index.flip_selected(rec);
        });

        self.rec_diff.status.run_if_changed(|o, n| {
            ptr.index.mutate_status(rec.pid, o.clone(), n.clone());
        });
    }
}

impl<'b, 'a: 'b> TableRecordPatch<'b, 'a> for PhotoRecordPatch<'b, 'a> {
    type Table = PhotoTable;
    fn new(rec: TableRecordMut<'a, Self::Table>, ptr: &'b TableRefMut<'a, Self::Table>) -> Self {
        let rec_ptr = unsafe { NonNull::new(std::mem::transmute(rec as *const _)).unwrap() };
        Self {
            commit_at_drop: true,
            rec_diff: PhotoRecordDiff::new(rec),
            rec: rec_ptr,
            ptr,
        }
    }
}
