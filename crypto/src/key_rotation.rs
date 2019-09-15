use std::{
    time::{
        Duration,
        Instant,
    },
    thread,
    sync::{
        Arc,
        RwLock,
        RwLockReadGuard,
        PoisonError,
        mpsc::{
            Sender,
            channel,
            RecvTimeoutError,
        },
    },
    ops::Deref,
};
use crate::algo::{Algo, SafeGenerateKey};

pub struct CurrAndLastKey<A: Algo> {
    pub algo: Arc<A>,
    pub last: A::Key,
    pub curr: A::Key,
}
impl<K: SafeGenerateKey + Clone + Send + Sync, A: Algo<Key = K>> CurrAndLastKey<A> {
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
    // TODO something like stuff
}

type GuardOrPoison<'a, T> = Result<RwLockReadGuard<'a, Arc<T>>, PoisonError<RwLockReadGuard<'a, Arc<T>>>> ;
pub struct KeyStore<A: Algo>(RwLock<Arc<CurrAndLastKey<A>>>);
impl<A: Algo> Deref for KeyStore<A> {
    type Target = RwLock<Arc<CurrAndLastKey<A>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<A: Algo> KeyStore<A> {
    pub fn curr_and_last(&self) -> GuardOrPoison<'_, CurrAndLastKey<A>> {
        self.read()
    }
}

pub struct KeyRotator<T: Algo> {
    pub key_store: Arc<KeyStore<T>>,
    kill_handle: Option<(Sender<()>, thread::JoinHandle<()>)>,
}

impl<K: SafeGenerateKey + Clone + Send + Sync, A: Algo<Key = K> + Send + Sync + 'static> KeyRotator<A> {
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
    pub fn cleanup(mut self) -> Result<(), Box<dyn std::any::Any + std::marker::Send + 'static>> {
        if let Some((tx, join_handle)) = self.kill_handle.take() {
            drop(tx);
            join_handle.join()
        } else {
            Ok(())
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

pub struct StaticKeyStore<A: Algo>(A, Arc<A::Key>);
impl<A: Algo> StaticKeyStore<A> {
    pub fn new(alg: A, k: A::Key) -> Self {
        Self(alg, Arc::new(k))
    }
}
