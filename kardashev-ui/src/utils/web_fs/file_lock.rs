use std::{
    collections::{
        hash_map,
        HashMap,
    },
    sync::{
        Arc,
        Weak,
    },
    time::Duration,
};

use parking_lot::Mutex;
use tokio::sync::{
    mpsc,
    OwnedRwLockReadGuard,
    OwnedRwLockWriteGuard,
    RwLock,
};

use crate::utils::{
    futures::{
        interval,
        spawn_local,
    },
    web_fs::database::InodeId,
};

#[derive(Clone, Debug)]
pub struct FileLocks {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Debug)]
struct Inner {
    locks: HashMap<InodeId, FileLock>,
}

impl FileLocks {
    pub fn new() -> Self {
        let inner = Arc::new(Mutex::new(Inner {
            locks: HashMap::new(),
        }));

        periodic_cleanup(&inner);

        Self { inner }
    }

    fn get_lock(&self, inode_id: InodeId) -> FileLock {
        let mut inner = self.inner.lock();

        match inner.locks.entry(inode_id) {
            hash_map::Entry::Occupied(occupied) => occupied.get().clone(),
            hash_map::Entry::Vacant(vacant) => {
                let lock = FileLock::new();
                vacant.insert(lock.clone());
                lock
            }
        }
    }

    pub async fn read(&self, inode_id: InodeId) -> FileLockReadGuard {
        let lock = self.get_lock(inode_id);
        lock.read().await
    }

    pub async fn write(&self, inode_id: InodeId) -> FileLockWriteGuard {
        let lock = self.get_lock(inode_id);
        lock.write().await
    }
}

#[derive(Clone, Debug)]
struct FileLock {
    lock: Arc<RwLock<()>>,
}

impl FileLock {
    fn new() -> Self {
        Self {
            lock: Arc::new(RwLock::new(())),
        }
    }

    async fn read(self) -> FileLockReadGuard {
        FileLockReadGuard {
            guard: self.lock.read_owned().await,
        }
    }

    async fn write(self) -> FileLockWriteGuard {
        FileLockWriteGuard {
            guard: self.lock.write_owned().await,
        }
    }

    fn is_locked(&self) -> bool {
        self.lock.try_write().is_ok()
    }
}

#[derive(Debug)]
pub struct FileLockReadGuard {
    guard: OwnedRwLockReadGuard<()>,
}

#[derive(Debug)]
pub struct FileLockWriteGuard {
    guard: OwnedRwLockWriteGuard<()>,
}

fn periodic_cleanup(inner: &Arc<Mutex<Inner>>) {
    static TASK_TX: Mutex<Option<mpsc::UnboundedSender<Weak<Mutex<Inner>>>>> = Mutex::new(None);

    let inner = Arc::downgrade(inner);

    let mut task_tx = TASK_TX.lock();
    if let Some(task_tx) = &*task_tx {
        task_tx.send(inner).expect("cleanup task dead");
    }
    else {
        let (tx, mut rx) = mpsc::unbounded_channel();
        *task_tx = Some(tx);

        spawn_local(async move {
            let mut inners = vec![inner];
            let mut interval = interval(Duration::from_secs(60));

            fn cleanup(inners: &mut Vec<Weak<Mutex<Inner>>>) {
                inners.retain_mut(|inner| {
                    if let Some(inner) = Weak::upgrade(&inner) {
                        let mut inner = inner.lock();
                        inner.locks.retain(|_, lock| lock.is_locked());
                        true
                    }
                    else {
                        false
                    }
                });
            }

            while !inners.is_empty() {
                tokio::select! {
                    _ = interval.tick() => cleanup(&mut inners),
                    inner_opt = rx.recv() => {
                        let Some(inner) = inner_opt else { break; };
                        inners.push(inner);
                    }
                }
            }

            // fixme: race condition
            let mut task_tx = TASK_TX.lock();
            *task_tx = None;
        });
    }
}
