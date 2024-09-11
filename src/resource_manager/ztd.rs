use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    str,
};

use anyhow::Context;
use zip::{read::ZipFile, ZipArchive};

// // TODO: Make some ZipFile and ZipArchive wrappers and add these functions to them
// fn read_file_from_zip(zip: &mut ZipArchive<BufReader<File>>, file_name: &str) -> anyhow::Result<Box<[u8]>> {
//     let mut file = zip.by_name(file_name).with_context(|| format!("Error finding file in archive: {}", file_name))?;

//     let mut file_buffer = vec![0u8; file.size() as usize].into_boxed_slice();

//     file.read_exact(&mut file_buffer).with_context(|| format!("Error reading file: {}", file_name))?;

//     Ok(file_buffer)
// }

// fn read_file_from_zip_to_string(zip: &mut ZipArchive<BufReader<File>>, file_name: &str) -> anyhow::Result<String> {
//     let buffer = read_file_from_zip(zip, file_name)?;

//     Ok(str::from_utf8(&buffer)
//         .with_context(|| format!("Error converting file {} to utf8", file_name))?
//         .to_string())
// }

// fn zip_file_to_string(mut zip: ZipFile) -> anyhow::Result<String> {
//     let mut buffer = vec![0u8; zip.size() as usize].into_boxed_slice();
//     zip.read_exact(&mut buffer).with_context(|| format!("Error reading file: {}", zip.name()))?;

//     Ok(str::from_utf8(&buffer)
//         .with_context(|| format!("Error converting file {} to utf8", zip.name()))?
//         .to_string())
// }

pub struct ZtdArchive {
    archive: ZipArchive<BufReader<File>>,
    archive_name: String,
    archive_path: PathBuf,
}

impl ZtdArchive {
    pub fn new(archive_path: &Path) -> anyhow::Result<Self> {
        let archive_name = archive_path.to_str().unwrap().to_string();
        let archive = ZipArchive::new(BufReader::new(
            File::open(&archive_path).with_context(|| format!("Failed to open archive {}", archive_path.display()))?,
        ))
        .with_context(|| format!("Failed to read archive {}", archive_path.display()))?;

        Ok(Self {
            archive,
            archive_name,
            archive_path: archive_path.to_path_buf(),
        })
    }

    pub fn name(&self) -> &str {
        &self.archive_name
    }

    pub fn by_name(&mut self, file_name: &str) -> anyhow::Result<ZtdFile> {
        let zip_file = self
            .archive
            .by_name(file_name)
            .with_context(|| format!("Error finding file in archive: {}", file_name))?;
        Ok(ZtdFile { inner: zip_file })
    }

    pub fn len(&self) -> usize {
        self.archive.len()
    }

    pub fn by_index(&mut self, index: usize) -> anyhow::Result<ZtdFile> {
        let zip_file = self
            .archive
            .by_index(index)
            .with_context(|| format!("Error finding file in archive at index: {}", index))?;
        Ok(ZtdFile { inner: zip_file })
    }

    pub fn file_names(&self) -> impl Iterator<Item = &str> {
        self.archive.file_names()
    }
}

// impl Iterator for ZtdArchive {
//     type Item<'a> = ZtdFile<'a>;

//     fn next(&mut self) -> Option<Self::Item> {
//         for i in 0..self.archive.len() {
//             let file = self.archive.by_index(i).unwrap();

//             if file.is_dir() {
//                 continue;
//             }
//             return Some(ZtdFile::new(file));
//         }
//         None
//     }
// }

pub struct ZtdFile<'a> {
    inner: ZipFile<'a>,
}

impl<'a> ZtdFile<'a> {
    pub fn new(inner: ZipFile<'a>) -> Self {
        Self { inner }
    }
}

impl ZtdFile<'_> {
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    pub fn read_exact(&mut self, buffer: &mut [u8]) -> anyhow::Result<()> {
        self.inner.read_exact(buffer).with_context(|| format!("Error reading file: {}", self.inner.name()))?;
        Ok(())
    }

    pub fn size(&self) -> u64 {
        self.inner.size()
    }

    pub fn is_dir(&self) -> bool {
        self.inner.is_dir()
    }

    pub fn read_to_string(&mut self) -> anyhow::Result<String> {
        let mut buffer = vec![0u8; self.inner.size() as usize].into_boxed_slice();
        self.inner
            .read_exact(&mut buffer)
            .with_context(|| format!("Error reading file: {}", self.inner.name()))?;

        Ok(str::from_utf8(&buffer)
            .with_context(|| format!("Error converting file {} to utf8", self.inner.name()))?
            .to_string())
    }
}

impl TryFrom<ZtdFile<'_>> for String {
    type Error = anyhow::Error;

    fn try_from(mut file: ZtdFile) -> Result<String, Self::Error> {
        let mut buffer = vec![0u8; file.size() as usize].into_boxed_slice();
        file.inner
            .read_exact(&mut buffer)
            .with_context(|| format!("Error reading file: {}", file.inner.name()))?;

        Ok(str::from_utf8(&buffer)
            .with_context(|| format!("Error converting file {} to utf8", file.inner.name()))?
            .to_string())
    }
}

// impl<E: std::convert::From<anyhow::Error>> TryInto<String> for ZtdFile<'_> {
//     type Error = anyhow::Error;

//     fn try_into(mut self) -> Result<String, Self::Error> {
//         let mut buffer = vec![0u8; self.inner.size() as usize].into_boxed_slice();
//         self.inner.read_exact(&mut buffer).with_context(|| format!("Error reading file: {}", self.inner.name()))?;

//         Ok::<std::string::String, E>(str::from_utf8(&buffer)
//             .with_context(|| format!("Error converting file {} to utf8", self.inner.name()))?
//             .to_string())
//     }
// }