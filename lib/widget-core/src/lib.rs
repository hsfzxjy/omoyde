use std::borrow::Cow;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Error(pub String);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

macro_rules! err {
    () => {
        err!("")
    };
    ($($tt:tt)+) => {
        Err(Error (
            format!(
                "[{}:{}:{}] {}",
                file!(),
                line!(),
                column!(),
                format!($($tt)*))
            )
        )?
    }
}

type Result<T> = std::result::Result<T, Error>;

macro_rules! checked_split {
    ($storage:ident, $offset:ident) => {{
        if ($offset > $storage.len()) {
            err!(
                "unexpected end: offset: {}, storage size: {}",
                $offset,
                $storage.len()
            )
        }
        Ok($storage.split_at($offset))
    }};
}

#[inline]
fn split<T>(storage: &[u8]) -> Result<(&[u8], &[u8])> {
    let len = std::mem::size_of::<T>();
    checked_split!(storage, len)
}

macro_rules! read_be {
    (u8char, $a: expr) => {{
        let (v, a) = split::<u8>($a)?;
        (char::from(v[0]), a)
    }};
    ($ty: tt, $a: expr) => {{
        let (v, a) = split::<$ty>($a)?;
        ($ty::from_be_bytes(v.try_into().unwrap()), a)
    }};
}

#[derive(Debug)]
struct MsgDatetime {
    base: u32,
    offset: i8,
}

impl PartialEq for MsgDatetime {
    fn eq(&self, other: &Self) -> bool {
        (self.base, self.offset) == (other.base, other.offset)
    }
}

impl PartialOrd for MsgDatetime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.base, self.offset).partial_cmp(&(other.base, other.offset))
    }
}

impl MsgDatetime {
    fn from_storage(storage: &[u8]) -> Result<(MsgDatetime, &[u8])> {
        let a = storage;
        let (base, a) = read_be!(u32, a);
        let (offset, a) = read_be!(i8, a);
        Ok((Self { base, offset }, a))
    }
}

impl Display for MsgDatetime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.base, self.offset)
    }
}

trait BorrowMsgDatetime {
    fn borrow_dt(&self) -> &MsgDatetime;
}

impl BorrowMsgDatetime for MsgDatetime {
    fn borrow_dt(&self) -> &MsgDatetime {
        &self
    }
}

impl BorrowMsgDatetime for MsgHeader {
    fn borrow_dt(&self) -> &MsgDatetime {
        &self.dt
    }
}

impl<'a> BorrowMsgDatetime for MsgItem<'a> {
    fn borrow_dt(&self) -> &MsgDatetime {
        &self.header.dt
    }
}

macro_rules! derive_dt_cmp {
    ($T:ty) => {
        impl<'a, S> PartialEq<S> for $T
        where
            S: BorrowMsgDatetime,
        {
            fn eq(&self, other: &S) -> bool {
                self.borrow_dt().eq(other.borrow_dt())
            }
        }
        impl<'a, S> PartialOrd<S> for $T
        where
            S: BorrowMsgDatetime,
        {
            fn partial_cmp(&self, other: &S) -> Option<Ordering> {
                self.borrow_dt().partial_cmp(other.borrow_dt())
            }
        }
    };
}

derive_dt_cmp!(MsgHeader);
derive_dt_cmp!(MsgItem<'a>);

const HEADER_SIZE: usize = 8;

#[derive(Debug)]
struct MsgHeader {
    ty: char,
    dt: MsgDatetime,
    text_len: u16,
}

impl MsgHeader {
    fn from_storage(storage: &[u8]) -> Result<(MsgHeader, &[u8])> {
        let a = storage;
        let (ty, a) = read_be!(u8char, a);
        let (dt, a) = MsgDatetime::from_storage(a)?;
        let (text_len, a) = read_be!(u16, a);
        Ok((Self { ty, dt, text_len }, a))
    }
}

impl<'a> Display for MsgHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "type: {}, dt: {}", self.ty, self.dt)
    }
}

pub struct MsgItem<'a> {
    inner: Cow<'a, [u8]>,
    header: MsgHeader,
}

impl<'a> MsgItem<'a> {
    #[inline]
    fn set_dt_offset(&mut self, offset: i8) {
        self.header.dt.offset = offset;
        self.inner.to_mut()[5] = offset as u8;
    }
    #[inline]
    fn from_storage(storage: &'a [u8]) -> Result<(MsgItem<'a>, &'a [u8])> {
        let (header, _) = MsgHeader::from_storage(storage)?;
        let record_len = HEADER_SIZE + header.text_len as usize;
        let (storage, rest) = checked_split!(storage, record_len)?;
        Ok((
            Self {
                inner: Cow::Borrowed(storage),
                header,
            },
            rest,
        ))
    }
    fn get_text_ptr(&self) -> &[u8] {
        &self.inner.as_ref()[HEADER_SIZE..]
    }
}

#[cfg(storage_encoding = "utf8")]
impl<'a> MsgItem<'a> {
    #[inline]
    fn from_utf8_storage(storage: &'a [u8]) -> Result<(MsgItem<'a>, &'a [u8])> {
        Self::from_storage(storage)
    }
    fn get_text(&self) -> String {
        String::from_utf8_lossy(self.get_text_ptr()).into_owned()
    }
}

#[cfg(storage_encoding = "utf16be")]
#[inline]
fn utf8_buffer_to_utf16be_words(buf: &[u8]) -> Vec<u16> {
    String::from_utf8_lossy(buf).encode_utf16().collect()
}

#[cfg(storage_encoding = "utf16be")]
#[inline]
fn write_utf16be_words(words: Vec<u16>, buf: &mut Vec<u8>) {
    for ch in words.into_iter() {
        buf.extend_from_slice(&ch.to_be_bytes())
    }
}

#[cfg(storage_encoding = "utf16be")]
impl<'a> MsgItem<'a> {
    #[inline]
    pub fn from_utf8_storage(storage: &'a [u8]) -> Result<(MsgItem<'a>, &'a [u8])> {
        let (mut header, rest) = MsgHeader::from_storage(storage)?;

        let text_len = header.text_len as usize;
        let (text_buf, rest) = checked_split!(rest, text_len)?;

        let utf16_words = utf8_buffer_to_utf16be_words(text_buf);
        let new_text_len = utf16_words.len() * 2;

        header.text_len = new_text_len as u16;

        let new_storage = {
            let mut s = Vec::<u8>::with_capacity(HEADER_SIZE + new_text_len);
            s.extend_from_slice(&storage[..HEADER_SIZE]);
            s[HEADER_SIZE - 2..HEADER_SIZE].copy_from_slice(&header.text_len.to_be_bytes());
            write_utf16be_words(utf16_words, &mut s);
            s
        };

        Ok((
            Self {
                inner: Cow::Owned(new_storage),
                header,
            },
            rest,
        ))
    }

    fn get_text(&self) -> String {
        let utf16_buf = self
            .get_text_ptr()
            .chunks(2)
            .map(|chunk| chunk.try_into().unwrap())
            .map(u16::from_be_bytes)
            .collect::<Vec<_>>();
        String::from_utf16_lossy(&utf16_buf)
    }
}

impl<'a> Display for MsgItem<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, text: {}", &self.header, self.get_text())
    }
}

#[inline]
fn parse_until<'a, T, Parse, Predicate>(
    storage: &'a [u8],
    parse: Parse,
    stop: Predicate,
) -> Result<(Vec<T>, &'a [u8])>
where
    T: 'a,
    Parse: Fn(&'a [u8]) -> Result<(T, &'a [u8])>,
    Predicate: Fn(&[u8]) -> bool,
{
    let mut buf = storage;
    let mut res = vec![];
    while !stop(buf) {
        let (item, new_buf) = parse(buf)?;
        buf = new_buf;
        res.push(item);
    }
    Ok((res, buf))
}

pub fn parse_items<'a>(storage: &'a [u8]) -> Result<Vec<MsgItem<'a>>> {
    parse_until(storage, MsgItem::from_storage, <[u8]>::is_empty).map(|x| x.0)
}

fn parse_mods(storage: &[u8]) -> Result<(Vec<MsgItem>, Vec<MsgDatetime>)> {
    let (adds, rest) = parse_until(storage, MsgItem::from_utf8_storage, |buf| buf[0] == 0)?;
    let (dels, _) = parse_until(&rest[1..], MsgDatetime::from_storage, <[u8]>::is_empty)?;
    Ok((adds, dels))
}

pub fn serialize_items<'a>(items: Vec<MsgItem<'a>>) -> Vec<u8> {
    let total_size = items
        .iter()
        .fold(0usize, |accum, item| accum + item.inner.len());
    let mut result = Vec::with_capacity(total_size);
    for item in items {
        result.extend_from_slice(&item.inner);
    }
    result
}

pub fn display_items<'a>(items: &Vec<MsgItem<'a>>) {
    for item in items {
        println!("{}", item);
    }
}

pub fn mod_items<'a>(items: &'a [u8], mods: &'a [u8]) -> Result<Vec<MsgItem<'a>>> {
    let mut old_items = parse_items(items)?.into_iter().peekable();
    let (adds, dels) = parse_mods(mods)?;
    let mut adds = adds.into_iter().peekable();
    let mut dels = dels.into_iter().peekable();
    let mut new_items = vec![];

    let mut nneg = 0;
    let mut npos = 0;
    let mut prev_dt_base = 0u32;
    let mut ended = false;

    loop {
        let mut pushed = false;
        match (adds.peek(), dels.peek(), old_items.peek()) {
            (None, None, None) => ended = true,
            (_, Some(del), None) => err!("invalid state: del: {:?} old_item: None", &del),
            (None, None, Some(_)) => {
                new_items.push(old_items.next().unwrap());
                pushed = true;
            }
            (Some(_), None, None) => {
                new_items.push(adds.next().unwrap());
                pushed = true;
            }
            (None, Some(del), Some(item)) => match item.partial_cmp(del) {
                Some(Ordering::Less) => {
                    new_items.push(old_items.next().unwrap());
                    pushed = true;
                }
                Some(Ordering::Equal) => {
                    old_items.next();
                    dels.next();
                }
                _ => err!(
                    "invalid state: del: {:?} old_item: {:?}",
                    &del,
                    &item.header.dt
                ),
            },
            (Some(add), del, Some(item)) => {
                if add < item {
                    new_items.push(adds.next().unwrap());
                    pushed = true;
                } else {
                    let old_item = old_items.next().unwrap();
                    let discard = match del {
                        Some(del) => &old_item == del,
                        _ => false,
                    };
                    if discard {
                        dels.next();
                    } else {
                        new_items.push(old_item);
                        pushed = true;
                    }
                };
            }
        }

        if pushed || ended {
            let MsgDatetime { base, offset } = new_items.last().unwrap().header.dt;
            let is_new_range = base != prev_dt_base;
            if is_new_range || ended {
                let mut right = new_items.len() - 1;
                if is_new_range {
                    right -= 1
                }
                for i in (1..=npos).rev() {
                    new_items[right].set_dt_offset(i as i8);
                    right -= 1
                }
                for i in 1..=nneg {
                    new_items[right].set_dt_offset(-i as i8);
                    right -= 1
                }
                prev_dt_base = base;
                nneg = 0;
                npos = 0;
                if offset >= 0 {
                    npos += 1
                } else {
                    nneg += 1
                };

                if is_new_range && ended {
                    new_items
                        .last_mut()
                        .unwrap()
                        .set_dt_offset(if offset >= 0 { 1 } else { -1 })
                }
            } else {
                if offset >= 0 {
                    npos += 1
                } else {
                    nneg += 1
                }
            }
        }
        if ended {
            break;
        }
    }

    Ok(new_items)
}
