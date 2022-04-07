pub mod fs {
    use crate::prelude::*;
    pub fn is_supported_image<P: AsRef<Path>>(path: P) -> bool {
        static SUPPORTED_EXTS: [&'static str; 2] = ["JPG", "JPEG"];
        let ext = if let Some(v) = path.as_ref().extension() {
            v.to_ascii_uppercase()
        } else {
            return false;
        };

        for x in SUPPORTED_EXTS.iter() {
            if OsStr::new(x) == ext {
                return true;
            }
        }
        return false;
    }
}

pub mod serde {

    use crate::prelude::*;

    pub trait TableIO<'a> {
        fn load_from_path<P: AsRef<Path>>(&'a mut self, path: P) -> Result<()>;
        fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    }

    impl<'a, T: serde::de::DeserializeOwned + Serialize + Table> TableIO<'a> for T {
        fn load_from_path<P: AsRef<Path>>(&'a mut self, path: P) -> Result<()> {
            let path = path.as_ref();
            if !path.exists() {
                fs::create_dir_all(path.parent().unwrap())?;
                return Ok(());
            }
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let desered: T = bincode::deserialize_from(reader)
                .map_err(|e| anyhow!("Error reading {}: {}", path.display(), e))?;
            mem::drop(mem::replace(self, desered));
            Ok(())
        }
        fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
            use atomicwrites::{AllowOverwrite, AtomicFile, Error as AtomicFileError};

            let modified = self.modified_flag().get();
            let path = path.as_ref();
            if !modified && path.exists() {
                debug!("Table unmodified, will not save to {}", path.display());
                return Ok(());
            }

            AtomicFile::new(path, AllowOverwrite)
                .write(|file| -> StdResult<(), _> {
                    let writer = BufWriter::new(file);
                    bincode::serialize_into(writer, self)
                })
                .map_err(|e| match e {
                    AtomicFileError::Internal(e) => e.into(),
                    AtomicFileError::User(e) => e.into(),
                })
        }
    }
}

pub mod exif {
    use crate::prelude::*;
    use exif;

    macro_rules! match_exif_value {
        ($variant: path) => {
            |field| {
                if let $variant(ref x) = field.value {
                    Some(x)
                } else {
                    None
                }
            }
        };
    }

    pub fn read_string(exif: &exif::Exif, tag: exif::Tag) -> Result<Option<String>> {
        Ok(exif
            .get_field(tag, exif::In::PRIMARY)
            .and_then(match_exif_value!(exif::Value::Ascii))
            .map(|vecs| String::from_utf8(vecs.concat()))
            .transpose()?)
    }

    pub fn read_u32(exif: &exif::Exif, tag: exif::Tag) -> Option<u32> {
        use exif::Value;
        exif.get_field(tag, exif::In::PRIMARY).and_then(|field| {
            field.value.get_uint(0).or_else(|| match field.value {
                Value::SLong(ref v) if v.len() > 0 => Some(v[0] as u32),
                Value::SShort(ref v) if v.len() > 0 => Some(v[0] as u32),
                Value::SByte(ref v) if v.len() > 0 => Some(v[0] as u32),
                _ => None,
            })
        })
    }

    pub fn read_datetime(exif: &exif::Exif) -> Result<Option<DateTime<Utc>>> {
        let ret = read_string(&exif, exif::Tag::DateTime)?
            .and_then(|time_str| NaiveDateTime::parse_from_str(&time_str, "%Y:%m:%d %H:%M:%S").ok())
            .and_then(|dt| {
                FixedOffset::east(8 * 60 * 60)
                    .from_local_datetime(&dt)
                    .single()
            })
            .map(DateTime::<Utc>::from);
        Ok(ret)
    }

    // https://stackoverflow.com/a/48488655/3278171
    pub fn read_dims_from_file<R: Seek + Read>(reader: &mut R) -> Result<(u32, u32)> {
        reader.rewind()?;
        let mut buf: Vec<u8> = vec![];
        reader.read_to_end(&mut buf)?;
        let mut offset = 0usize;
        while offset < buf.len() {
            while buf[offset] == 0xff {
                offset += 1;
            }
            let marker = buf[offset];
            offset += 1;
            match marker {
                0xd8 => continue, // SOI
                0xd9 => break,    // EOI
                x if 0xd0 <= x && x <= 0xd7 => continue,
                0x01 => continue, // TEM
                _ => (),
            }
            let len = ((buf[offset] as usize) << 8) | buf[offset + 1] as usize;
            offset += 2;

            if marker == 0xc0 || marker == 0xc2 {
                let h = ((buf[offset + 1] as u32) << 8) | (buf[offset + 2] as u32);
                let w = ((buf[offset + 3] as u32) << 8) | (buf[offset + 4] as u32);
                return Ok((w, h));
            }

            offset += len - 2;
        }
        Ok((0, 0))
    }

    pub fn read_dims<R: Seek + Read>(exif: &exif::Exif, reader: &mut R) -> Result<(u32, u32)> {
        let width = read_u32(&exif, exif::Tag::PixelXDimension)
            .or_else(|| read_u32(&exif, exif::Tag::ImageWidth));
        let height = read_u32(&exif, exif::Tag::PixelYDimension)
            .or_else(|| read_u32(&exif, exif::Tag::ImageLength));
        if let (Some(w), Some(h)) = (width, height) {
            return Ok((w, h));
        };
        read_dims_from_file(reader)
    }
    pub fn read_orientation(exif: &exif::Exif) -> u32 {
        read_u32(&exif, exif::Tag::Orientation).or(Some(1)).unwrap()
    }
}

pub mod math {
    use crate::prelude::*;

    fn gcd(mut x: u32, mut y: u32) -> u32 {
        let mut r;
        while x != 0 {
            r = y % x;
            y = x;
            x = r;
        }
        y
    }

    pub fn limit_ratio(mut x: u32, mut y: u32, max: u32) -> Result<(u32, u32)> {
        let swapped = if x > y {
            let tmp = x;
            x = y;
            y = tmp;
            true
        } else {
            false
        };

        // now x <= y
        let gcd = gcd(x, y);
        x /= gcd;
        y /= gcd;
        if x == 1 {
            bail!("fail to limit ratio for {} {} with max {}", x, y, max)
        }

        if y > max {
            let (mut p0, mut q0, mut p1, mut q1) = (0u32, 1u32, 1u32, 0u32);
            let mut n = x;
            let mut d = y;
            loop {
                if d == 0 {}
                let a = n / d;
                let q2 = q0 + a * q1;
                if q2 > max {
                    break;
                }
                let (np0, nq0, np1, nq1) = (p1, q1, p0 + a * p1, q2);
                p0 = np0;
                q0 = nq0;
                p1 = np1;
                q1 = nq1;
                let (nn, nd) = (d, n - a * d);
                n = nn;
                d = nd;
            }
            let k = (max - q0) / q1;
            let candidate1 = (p0 + k * p1, (q0 + k * q1));
            let bound1 = candidate1.0 as f32 / candidate1.1 as f32;
            let candidate2 = (p1, q1);
            let bound2 = candidate2.0 as f32 / candidate2.1 as f32;
            let val = x as f32 / y as f32;
            let result = if (bound1 - val).abs() < (bound2 - val).abs() {
                candidate1
            } else {
                candidate2
            };
            x = result.0;
            y = result.1;
        }
        Ok(if swapped { (y, x) } else { (x, y) })
    }

    #[test]
    fn test_limit_ratio() -> Result<()> {
        let x = 3141592653u32;
        let y = 1000000000u32;
        assert_eq!(limit_ratio(x, y, 10000)?, (355, 113));
        assert_eq!(limit_ratio(y, x, 10000)?, (113, 355));
        assert_eq!(limit_ratio(x, y, 355)?, (355, 113));
        assert_eq!(limit_ratio(x, y, 354)?, (333, 106));
        assert_eq!(limit_ratio(3840, 5120, 255)?, (3, 4));
        Ok(())
    }
}

pub mod sync {
    use serde::de::Deserialize;
    use serde::ser::Serialize;
    use std::fmt;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    #[derive(Debug, Default)]
    pub struct AtomicFlag(AtomicBool);

    impl AtomicFlag {
        pub fn new() -> Self {
            Self(AtomicBool::new(false))
        }

        pub fn set(&self) {
            self.0.store(true, Ordering::Relaxed);
        }

        pub fn get(&self) -> bool {
            self.0.load(Ordering::Relaxed)
        }
    }

    #[derive(Debug)]
    pub struct AtomicCounter(AtomicU32);

    impl AtomicCounter {
        pub fn new(v: u32) -> Self {
            Self(AtomicU32::new(v))
        }

        #[inline(always)]
        pub fn get_and_incr(&self) -> u32 {
            self.0.fetch_add(1, Ordering::Relaxed)
        }

        #[inline(always)]
        pub fn get(&self) -> u32 {
            self.0.load(Ordering::Relaxed)
        }
    }

    struct AtomicCounterVisitor;
    impl<'de> serde::de::Visitor<'de> for AtomicCounterVisitor {
        type Value = AtomicCounter;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("counter")
        }
        fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> {
            Ok(Self::Value::new(v))
        }
    }

    impl Serialize for AtomicCounter {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_u32(self.get())
        }
    }

    impl<'de> Deserialize<'de> for AtomicCounter {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_u32(AtomicCounterVisitor)
        }
    }
}

#[macro_use]
pub mod functional {
    use paste::paste;
    use std::collections::HashSet;

    pub trait Accumulable {
        fn init() -> Self;
        fn accum(x: Self, y: Self) -> Self;
    }

    impl<T: Eq + std::hash::Hash + Clone> Accumulable for HashSet<T> {
        fn init() -> Self {
            HashSet::new()
        }
        fn accum(mut x: Self, y: Self) -> Self {
            x.extend(y);
            x
        }
    }

    impl<T> Accumulable for Vec<T> {
        fn init() -> Self {
            vec![]
        }
        fn accum(mut x: Self, mut y: Self) -> Self {
            x.append(&mut y);
            x
        }
    }

    macro_rules! tuple_impl {
        ( $($T: ident),+ ) => {
            #[allow(non_snake_case)]
            impl< $( $T, )+ > Accumulable for ( $( $T, )+ ) where
            $( $T: Accumulable, )+ {
                fn init() -> Self {
                    ($(
                        <$T as Accumulable>::init(),
                    )+)
                }
                fn accum(x: Self, y: Self) -> Self {
                    let ( $(paste!{[<x $T>]},)+ ) = x;
                    let ( $(paste!{[<y $T>]},)+ ) = y;
                    ($(
                        Accumulable::accum(paste!{[<x $T>]}, paste!{[<y $T>]}),
                    )+)
                }
            }
        };
    }

    tuple_impl!(A);
    tuple_impl!(A, B);
    tuple_impl!(A, B, C);

    pub trait FoldInto {
        type Target;

        fn fold_into(self) -> Self::Target;
    }

    impl<I, T> FoldInto for I
    where
        I: Iterator<Item = T>,
        T: Accumulable,
    {
        type Target = T;

        fn fold_into(self) -> T {
            self.fold(<T as Accumulable>::init(), <T as Accumulable>::accum)
        }
    }
}

pub mod tabled {
    use std::fmt::Display;
    use tabled::{Alignment, Full, Modify, Style, Table, Tabled};

    pub fn display_option<T: Display>(o: &Option<T>) -> String {
        match o {
            Some(s) => format!("{}", s),
            None => "<NONE>".into(),
        }
    }

    pub fn print_table<I, T>(iter: I)
    where
        T: Tabled,
        I: IntoIterator<Item = T>,
    {
        let content = Table::new(iter)
            .with(Style::blank())
            .with(Modify::new(Full).with(Alignment::left()))
            .to_string();
        print!("{}", content);
    }
}
