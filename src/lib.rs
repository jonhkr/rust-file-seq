use std::fs;
use std::io::{Error, ErrorKind, Read};
use std::path::{Path, PathBuf};

use log::warn;

#[derive(Debug)]
pub struct FileSeq {
    path_1: PathBuf,
    path_2: PathBuf,
}

impl FileSeq {
    pub fn new<P: AsRef<Path>>(store_dir: P, initial_value: u64) -> std::io::Result<Self> {
        let store_path = store_dir.as_ref();

        fs::create_dir_all(store_path)?;
        let store_path_buf = store_path.to_path_buf();
        let path_1 = store_path_buf.join("_1.seq");
        let path_2 = store_path_buf.join("_2.seq");

        let seq = Self { path_1, path_2 };

        seq.initialize_if_necessary(initial_value)?;

        Ok(seq)
    }

    fn initialize_if_necessary(&self, initial_value: u64) -> std::io::Result<()> {
        if fs::metadata(&self.path_1).is_ok() || fs::metadata(&self.path_2).is_ok() {
            Ok(())
        } else {
            self.write(initial_value)
        }
    }

    pub fn delete(&self) -> std::io::Result<()> {
        fs::remove_file(&self.path_1)?;
        fs::remove_file(&self.path_2)
    }

    pub fn get_and_increment(&self, increment: u64) -> std::io::Result<u64> {
        let value = self.read()?;
        self.write(value + increment)?;
        Ok(value)
    }

    pub fn increment_and_get(&self, increment: u64) -> std::io::Result<u64> {
        let value = self.get_and_increment(increment)?;
        Ok(value + increment)
    }

    pub fn value(&self) -> std::io::Result<u64> {
        self.read()
    }

    fn read(&self) -> std::io::Result<u64> {
        let mut value1: Option<u64> = None;
        if fs::metadata(&self.path_1).is_ok() {
            let value = self.read_from_path(&self.path_1)?;
            value1 = Some(value);
        }

        let mut value2: Option<u64> = None;
        if fs::metadata(&self.path_2).is_ok() {
            value2 = self.read_from_path(&self.path_2).ok();
        }

        match value2 {
            Some(v2) => match value1 {
                Some(v1) => {
                    if v2 > v1 {
                        Ok(v2)
                    } else {
                        warn!("Latest sequence value is smaller than backup, using backup.");
                        fs::remove_file(&self.path_2).ok();
                        Ok(v1)
                    }
                }
                None => Ok(v2),
            },
            None => {
                fs::remove_file(&self.path_2).ok();

                match value1 {
                    Some(v1) => Ok(v1),
                    None => Err(Error::new(
                        ErrorKind::InvalidData,
                        "Looks like both backup and latest sequence files are corrupted.",
                    )),
                }
            }
        }
    }

    fn read_from_path<P: AsRef<Path>>(&self, path: P) -> std::io::Result<u64> {
        let mut buff = [0; 8];
        let mut f = fs::File::open(path.as_ref())?;
        f.read_exact(&mut buff)?;
        let value = u64::from_be_bytes(buff);
        Ok(value)
    }

    fn write(&self, value: u64) -> std::io::Result<()> {
        if fs::metadata(&self.path_2).is_ok() {
            fs::rename(&self.path_2, &self.path_1)?;
        }
        self.write_to_path(&self.path_2, value)
    }

    fn write_to_path<P: AsRef<Path>>(&self, path: P, value: u64) -> std::io::Result<()> {
        fs::write(path.as_ref(), value.to_be_bytes())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    use rand::RngCore;

    use crate::FileSeq;

    pub fn tmpdir() -> PathBuf {
        let p = env::temp_dir();
        let mut r = rand::thread_rng();
        let ret = p.join(&format!("file-seq-{}", r.next_u32()));
        fs::create_dir(&ret).unwrap();
        ret
    }

    #[test]
    fn should_store_initial_seq_correctly() {
        let dir = tmpdir();
        let seq = FileSeq::new(&dir, 1).unwrap();
        assert!(std::fs::metadata(dir).is_ok());
        assert!(std::fs::metadata(seq.path_2).is_ok());
    }

    #[test]
    fn should_cycle_seq_files() {
        let dir = tmpdir();
        let seq = FileSeq::new(&dir, 1).unwrap();
        assert!(std::fs::metadata(dir).is_ok());
        assert!(std::fs::metadata(&seq.path_2).is_ok());
        let path_2_value = std::fs::read(&seq.path_2).unwrap();
        seq.increment_and_get(1).unwrap();
        let path_1_value = std::fs::read(&seq.path_1).unwrap();
        assert_eq!(path_2_value, path_1_value);
    }

    #[test]
    fn should_delete() {
        let dir = tmpdir();
        let seq = FileSeq::new(&dir, 1).unwrap();
        assert!(std::fs::metadata(dir).is_ok());
        assert!(std::fs::metadata(&seq.path_2).is_ok());
        seq.increment_and_get(1).unwrap();
        seq.delete().unwrap();
        assert!(!std::fs::metadata(&seq.path_1).is_ok());
        assert!(!std::fs::metadata(&seq.path_2).is_ok());
    }

    #[test]
    fn should_increment_and_get() {
        let dir = tmpdir();
        let seq = FileSeq::new(dir, 1).unwrap();
        let prev_value = seq.value().unwrap();
        let curr_value = seq.increment_and_get(1).unwrap();
        assert_eq!(prev_value + 1, curr_value);
        assert_eq!(curr_value, seq.value().unwrap());
    }

    #[test]
    fn should_get_and_increment() {
        let dir = tmpdir();
        let seq = FileSeq::new(dir, 1).unwrap();
        let prev_value = seq.value().unwrap();
        let curr_value = seq.get_and_increment(1).unwrap();
        assert_eq!(prev_value, curr_value);
        assert_eq!(curr_value + 1, seq.value().unwrap())
    }
}
