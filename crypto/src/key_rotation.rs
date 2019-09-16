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
};
use crate::algo::{Algo, SafeGenerateKey};

mod stable {
    use crate::algo::Algo;
    pub struct KeyStore<A: Algo>(A, A::Key);
    impl<A: Algo> KeyStore<A> {
        pub fn new(alg: A, k: A::Key) -> Self {
            Self(alg, k)
        }
        pub fn key(&self) -> &A::Key {
            &self.1
        }
        pub fn alg(&self) -> &A {
            &self.0
        }
    }
}
pub use stable::KeyStore as StableKeyStore;

mod rotating {
    use std::sync::Arc;
    use crate::algo::{Algo, SafeGenerateKey};

    pub struct KeyStore<A: Algo> {
        pub algo: Arc<A>,
        pub last: Arc<A::Key>,
        pub curr: Arc<A::Key>,
    }
    impl<K: SafeGenerateKey + Clone + Send + Sync, A: Algo<Key = K>> KeyStore<A> {
        pub(super) fn new(alg: A) -> Self {
            let key = Arc::new(A::Key::generate(alg.key_settings()));
            Self {
                algo: Arc::new(alg),
                last: Arc::clone(&key),
                curr: key,
            }
        }
        pub(super) fn involute(&self) -> Arc<Self> {
            Arc::new(Self {
                algo: Arc::clone(&self.algo),
                last: Arc::clone(&self.curr),
                curr: Arc::new(A::Key::generate(self.algo.key_settings())),
            })
        }
    }
}
pub use rotating::KeyStore as RotatingKeyStore;

pub type RotatingKeyFixture<A> = Arc<RwLock<Arc<RotatingKeyStore<A>>>>;
pub trait Generational {
    type Error;
    type Datum;
    fn advance_generation(&self) -> Result<&Self, Self::Error>;
    fn get_store(&self) -> Result<Self::Datum, Self::Error>;
}
impl<K: SafeGenerateKey + Clone + Send + Sync, A: Algo<Key = K> + Send + Sync + 'static> Generational for RotatingKeyFixture<A> {
    type Error = Arc<RotatingKeyStore<A>>;
    type Datum = Arc<RotatingKeyStore<A>>;
    fn advance_generation(&self) -> Result<&Self, Self::Error> {
        let mut key_store = self.write()
            .map_err(|rwlg| Arc::clone(&*rwlg.into_inner()))?;
        *key_store = key_store.involute();
        Ok(self)
    }
    fn get_store(&self) -> Result<Self::Datum, Self::Error> {
        self.read()
            .map(|rwlg| Arc::clone(&*rwlg))
            .map_err(|poisoned_rwlg| Arc::clone(&*poisoned_rwlg.into_inner()))
    }
}

#[must_use]
pub struct KeyRotator<A: Algo> {
    pub key_store: RotatingKeyFixture<A>,
    kill_handle: Option<(Sender<()>, thread::JoinHandle<()>)>,
}

impl<K: SafeGenerateKey + Clone + Send + Sync, A: Algo<Key = K> + Send + Sync + 'static> KeyRotator<A> {
    pub fn init(alg: A, period_between_rotation: Option<Duration>) -> Self {
        let local_copy = Arc::new(RwLock::new(Arc::new(RotatingKeyStore::new(alg))));
        let remote_copy = Arc::clone(&local_copy);

        let (tx, rx) = channel();
        const TWO_HOURS_IN_SECONDS: u64 = 2 * 60 * 60;
        let period_between_rotation = period_between_rotation.unwrap_or(Duration::from_secs(TWO_HOURS_IN_SECONDS));

        let handle = thread::spawn(move || {
            let key_store_fixture = remote_copy;
            loop {
                let deadline = Instant::now() + period_between_rotation;
                if let Err(_) = key_store_fixture.advance_generation() {
                    use log::error;
                    // TODO recover or exit everything gracefully
                    error!("Key rotation thread crashed.");
                    panic!("Thread crashed!");
                }
                match rx.recv_timeout(Instant::now() - deadline) {
                    Err(RecvTimeoutError::Disconnected) => break,
                    _ => ()// continue if nothing
                }
            }
            ()
        });

        Self {
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
            use log::error;
            error!("Attempted to drop KeyRotation. Please call `cleanup` instead. Will now panic.");
            panic!("Attempted to drop KeyRotation. Please call `cleanup` instead.");
        }
    }
}

impl<A: Algo> KeyRotator<A> {
    pub fn get_key_fixture(&self) -> RotatingKeyFixture<A> {
        Arc::clone(&self.key_store)
    }
}
// TODO isolate Rocket compatability in a feature flag.
impl<A: Algo> KeyRotator<A> {
    pub fn get_rocket_managed_state(&self) -> RotatingKeyFixture<A> {
        Arc::clone(&self.key_store)
    }
}

