use std::{fs::File, io};

use fd_lock::{RwLock, RwLockWriteGuard};

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("File lock is already locked")]
    AlreadyLocked(#[from] io::Error),
}

pub struct Lock(RwLock<File>);

impl Lock {
    pub fn init() -> Result<Self, LockError> {
        let base_dirs = directories::BaseDirs::new().unwrap();

        #[cfg(target_os = "linux")]
        let dir = base_dirs.runtime_dir().unwrap();
        #[cfg(not(target_os = "linux"))]
        let dir = base_dirs.cache_dir();

        let lock_path = dir.join("fauxchat.lock");

        let lock = RwLock::new(File::create(lock_path)?);

        Ok(Self(lock))
    }

    pub fn try_lock(&mut self) -> Result<RwLockWriteGuard<'_, File>, LockError> {
        use std::{io::Write, process};

        let mut write_lock = self.0.try_write()?;
        write!(write_lock, "{}", process::id())?;

        Ok(write_lock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_locks() -> Result<(), LockError> {
        let mut first_lock = Lock::init()?;
        println!("Locking first lock");
        let _first_guard = first_lock.try_lock()?;

        let mut second_lock = Lock::init()?;
        println!("Locking second lock");
        let second_guard = second_lock.try_lock();

        // Locking an already locked guard should fail
        assert!(second_guard.is_err());

        Ok(())
    }
}
