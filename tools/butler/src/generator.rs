use crate::db;
use crate::prelude::*;
use crate::util;
use atomicwrites::{AllowOverwrite, AtomicFile};
use image::io::Reader as ImageReader;
use image::jpeg::JpegEncoder;

struct EncodeAsOriginal {
    subdir: &'static str,
}

struct EncodeAsThumbnail {
    subdir: &'static str,
    height: u32,
    width: u32,
}

enum EncoderParameter {
    Original(EncodeAsOriginal),
    Minify(EncodeAsThumbnail),
}

impl EncoderParameter {
    fn subdir(&self) -> &'static str {
        use EncoderParameter::*;
        match self {
            Original(EncodeAsOriginal { subdir }) => subdir,
            Minify(EncodeAsThumbnail { subdir, .. }) => subdir,
        }
    }
}

const ENCODER_PARAMETERS: [EncoderParameter; 3] = [
    EncoderParameter::Minify(EncodeAsThumbnail {
        subdir: "s",
        height: 100,
        width: 100,
    }),
    EncoderParameter::Minify(EncodeAsThumbnail {
        subdir: "m",
        height: 800,
        width: 800,
    }),
    EncoderParameter::Original(EncodeAsOriginal { subdir: "source" }),
];

#[derive(Debug)]
pub struct PhotoGenerator {
    pid: u32,
    dst_dir: PathBuf,
    quality: u8,
    force: bool,
    source_path: PathBuf,
    metadata: db::PhotoMetadata,
    commit_time: DateTime<Utc>,
    exif_time: DateTime<Utc>,
    image: Option<image::DynamicImage>,
}

pub struct CompressedMeta {
    pid: u32,
    timestamp: u32,
    h: u8,
    w: u8,
}

pub fn write_bins<P: AsRef<Path>>(mut metas: Vec<CompressedMeta>, path: P) -> Result<()> {
    metas.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    use byteorder::{BigEndian, WriteBytesExt};
    let mut writer = BufWriter::new(File::create(path)?);
    for meta in metas {
        writer.write_uint::<BigEndian>(meta.pid.into(), 3)?;
        writer.write_u32::<BigEndian>(meta.timestamp)?;
        writer.write_u8(meta.h)?;
        writer.write_u8(meta.w)?;
    }
    Ok(())
}

impl PhotoGenerator {
    pub fn new<P: AsRef<Path>>(
        entry: &mut db::DBPhotoEntry,
        dst_dir: P,
        force: bool,
        quality: u8,
    ) -> Result<Self> {
        Ok(Self {
            force,
            quality,
            dst_dir: dst_dir.as_ref().into(),
            pid: entry.pid,
            source_path: entry.location.filepath().into(),
            metadata: entry.metadata.clone(),
            exif_time: entry
                .metadata
                .etime
                .ok_or_else(|| anyhow!("{} does not have an EXIF time", entry.pid))?,
            commit_time: entry
                .commit_time
                .ok_or_else(|| anyhow!("{} does not have a commit time", entry.pid))?,
            image: None,
        })
    }
    fn get_compressed_meta(self) -> Result<CompressedMeta> {
        use db::PhotoOrientation::*;
        let (h, w) = match (
            self.metadata.orientation,
            self.metadata.height,
            self.metadata.width,
        ) {
            (D0 | D180, h, w) => (h, w),
            (_, h, w) => (w, h),
        };
        if h == 0 || w == 0 {
            bail!("{} has invalid dimensions", self.pid);
        }
        let (h, w) = util::math::limit_ratio(h, w, 255)?;
        Ok(CompressedMeta {
            pid: self.pid,
            timestamp: self.exif_time.timestamp() as u32,
            h: h as u8,
            w: w as u8,
        })
    }
    pub fn generate(mut self) -> Result<CompressedMeta> {
        for param in &ENCODER_PARAMETERS {
            self.generate_for(param)?;
        }
        Ok(self.get_compressed_meta()?)
    }
    fn rotate_image(&self, img: image::DynamicImage) -> image::DynamicImage {
        use db::PhotoOrientation::*;
        match self.metadata.orientation {
            D270 => img.rotate270(),
            D180 => img.rotate180(),
            D90 => img.rotate90(),
            D0 => img,
        }
    }
    fn image_ref(&mut self) -> Result<&image::DynamicImage> {
        if self.image.is_none() {
            let img = ImageReader::open(&self.source_path)?.decode()?;
            let img = self.rotate_image(img);
            self.image = Some(img);
        }
        Ok(self.image.as_ref().unwrap())
    }
    fn generate_for(&mut self, param: &EncoderParameter) -> Result<()> {
        let dst_dir = self.dst_dir.join(param.subdir());
        fs::create_dir_all(&dst_dir)?;
        let dst_file = dst_dir.join(format!("{}.jpg", self.pid));
        if !self.force && dst_file.exists() {
            let mtime: DateTime<Utc> = dst_file.metadata()?.modified()?.into();
            if mtime >= self.commit_time {
                println!("Already generated: {}", dst_file.display());
                return Ok(());
            }
        }

        println!("Generating {}...", dst_file.display());
        use EncoderParameter::*;
        match param {
            Original(v) => self.original(dst_file, v),
            Minify(v) => self.minify(dst_file, v),
        }
    }
    fn minify(&mut self, dst: PathBuf, param: &EncodeAsThumbnail) -> Result<()> {
        let thumbnail = self.image_ref()?.thumbnail(param.width, param.height);

        AtomicFile::new(dst, AllowOverwrite)
            .write(|file| {
                let mut encoder = JpegEncoder::new_with_quality(file, self.quality);
                encoder.encode_image(&thumbnail)
            })
            .map_err(Error::new)
    }

    fn original(&mut self, dst: PathBuf, _param: &EncodeAsOriginal) -> Result<()> {
        let _ = std::fs::remove_file(&dst);
        std::os::unix::fs::symlink(&self.source_path, &dst).or_else(|err| match err.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(err.into()),
        })
    }
}
