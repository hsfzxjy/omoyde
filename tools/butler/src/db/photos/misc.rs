use crate::prelude::*;
use crate::util;

pub type FileHash = u64;
pub type PID = u32;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum PhotoOrientation {
    D0,
    D90,
    D180,
    D270,
}

impl From<u32> for PhotoOrientation {
    fn from(v: u32) -> Self {
        use PhotoOrientation::*;
        match v {
            0 | 1 => D0,
            3 => D180,
            6 => D90,
            8 => D270,
            _ => panic!("unknown orientation value {}", v),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PhotoMetadata {
    // fields for checking modification
    pub ctime: DateTime<Utc>,
    pub mtime: DateTime<Utc>,
    pub file_length: u64,
    // EXIF fields
    pub etime: Option<DateTime<Utc>>, // EXIF time
    pub width: u32,
    pub height: u32,
    pub orientation: PhotoOrientation,
}

impl PhotoMetadata {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        use util::exif::{read_datetime, read_dims, read_orientation};

        let filepath = path.as_ref();
        let metadata = filepath.metadata()?;

        let file = File::open(&filepath)?;
        let mut reader = BufReader::new(file);

        let exif_reader = exif::Reader::new();
        let exif = exif_reader.read_from_container(&mut reader);

        let (etime, orientation, width, height) = if let Ok(exif) = exif {
            let etime = read_datetime(&exif)?;

            let (width, height) = read_dims(&exif, &mut reader)?;
            let orientation = PhotoOrientation::from(read_orientation(&exif));
            (etime, orientation, width, height)
        } else {
            let (width, height) = util::exif::read_dims_from_file(&mut reader)?;
            (None, PhotoOrientation::D0, width, height)
        };

        Ok(Self {
            ctime: DateTime::from(metadata.created()?),
            mtime: DateTime::from(metadata.modified()?),
            file_length: metadata.len(),
            etime,
            orientation,
            width,
            height,
        })
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PhotoEntryStatus {
    Committed,
    CommittedButMissing,
    CommittedButModified,
    Uncommitted,
}

