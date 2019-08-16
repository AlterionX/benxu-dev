use std::{
    time::{
        Duration,
        Instant,
    },
    thread,
    sync::{
        Arc,
        RwLock,
        mpsc::{
            Sender,
            channel,
            RecvTimeoutError,
        },
    },
    ops::Deref,
};
use crate::crypto::algo::{Algo, Key};

pub struct CurrAndLastKey<A: Algo> {
    pub algo: Arc<A>,
    pub last: A::Key,
    pub curr: A::Key,
}
impl<A: Algo> CurrAndLastKey<A> {
    fn new(alg: A) -> Self {
        let key = A::Key::generate(alg.key_settings());
        Self {
            algo: Arc::new(alg),
            last: key.clone(),
            curr: key,
        }
    }
    fn progress(&self) -> Self {
        Self {
            algo: Arc::clone(&self.algo),
            last: self.curr.clone(),
            curr: A::Key::generate(self.algo.key_settings()),
        }
    }
}

pub struct KeyStore<T: Algo>(RwLock<Arc<CurrAndLastKey<T>>>);
impl<T: Algo> Deref for KeyStore<T> {
    type Target = RwLock<Arc<CurrAndLastKey<T>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct KeyRotator<T: Algo> {
    pub key_store: Arc<KeyStore<T>>,
    kill_handle: Option<(Sender<()>, thread::JoinHandle<()>)>,
}

impl<A: Algo + Send + Sync + 'static> KeyRotator<A> {
    pub fn init(alg: A, period_between_rotation: Option<Duration>) -> KeyRotator<A> {
        let local_copy = Arc::new(KeyStore(RwLock::new(Arc::new(CurrAndLastKey::new(alg)))));

        let remote_copy = Arc::clone(&local_copy);
        let (tx, rx) = channel();
        const TWO_HOURS_IN_SECONDS: u64 = 2 * 60 * 60;
        let period_between_rotation = period_between_rotation.unwrap_or(Duration::from_secs(TWO_HOURS_IN_SECONDS));

        let handle = thread::spawn(move || {
            loop {
                let deadline = Instant::now() + period_between_rotation;
                if let Ok(mut keys) = (*remote_copy).write() {
                    *keys = Arc::new(keys.progress());
                } else {
                    // TODO recover or exit everything gracefully
                    panic!("Thread crashed!");
                }
                match rx.recv_timeout(Instant::now() - deadline) {
                    Err(RecvTimeoutError::Disconnected) => break,
                    _ => ()// continue if nothing
                }
            }
            ()
        });

        KeyRotator {
            key_store: local_copy,
            kill_handle: Some((tx, handle)),
        }
    }
    pub fn cleanup(mut self) {
        if let Some((tx, join_handle)) = self.kill_handle.take() {
            drop(tx);
            join_handle.join();
        }
    }
}
impl<T: Algo> Drop for KeyRotator<T> {
    fn drop(&mut self) {
        if self.kill_handle.is_some() {
            // log error
        }
    }
}

