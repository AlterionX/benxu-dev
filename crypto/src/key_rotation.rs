//! Structs and methods used for storing keys and auto-cycling keys based on a periodic functions.
//!
//! TOOD: Make sure to allow for a signal to be sent externally from the local machine to manually
//! cycle keys.

use crate::algo::{Algo, SafeGenerateKey};
use std::{
    sync::{
        mpsc::{channel, RecvTimeoutError, Sender},
        Arc, RwLock,
    },
    thread,
    time::{Duration, Instant},
};

/// A stable key store. Not very interesting.
mod stable {
    use crate::algo::Algo;

    /// Maintains an algorithm and its key.
    pub struct KeyStore<A: Algo>(
        /// An algorithm.
        A,
        /// A key matching the algorithm.
        A::Key,
    );
    impl<A: Algo> KeyStore<A> {
        /// Uses the provided algo and key to create itself.
        pub fn new(alg: A, k: A::Key) -> Self {
            Self(alg, k)
        }
        /// Returns a reference to the key.
        pub fn key(&self) -> &A::Key {
            &self.1
        }
        /// Returns a reference to the algo.
        pub fn alg(&self) -> &A {
            &self.0
        }
    }
}
pub use stable::KeyStore as StableKeyStore;

/// A rotating key store/one to be used with the [`KeyRotator`](crate::KeyRotator).
mod rotating {
    use crate::algo::{Algo, SafeGenerateKey};
    use std::sync::Arc;
    /// A Send/Sync key store that keeps the last two keys.
    pub struct KeyStore<A: Algo> {
        /// A pointer to the algorithm.
        pub algo: Arc<A>,
        /// A pointer to the previous key. TODO: Rename as `prev`.
        pub last: Arc<A::Key>,
        /// A pointer to the current key.
        pub curr: Arc<A::Key>,
    }
    impl<K: SafeGenerateKey + Clone + Send + Sync, A: Algo<Key = K>> KeyStore<A> {
        /// Creates a new [`KeyStore`], generating the initial two keys.
        pub(super) fn new(alg: A) -> Self {
            let key = Arc::new(A::Key::safe_generate(alg.key_settings()));
            Self {
                algo: Arc::new(alg),
                last: Arc::clone(&key),
                curr: key,
            }
        }
        /// Undertake involution. AKA progress the current key to the last key and generate a new
        /// key.
        pub(super) fn involute(&self) -> Arc<Self> {
            Arc::new(Self {
                algo: Arc::clone(&self.algo),
                last: Arc::clone(&self.curr),
                curr: Arc::new(A::Key::safe_generate(self.algo.key_settings())),
            })
        }
        /// Attempt to use the current key, then the previous key. The function `attempt` takes in
        /// a key, and an optional result, which is populated if the first attempt fails.
        pub fn attempt_with_retry<T, E, F>(&self, attempt: &mut F) -> Result<T, E>
        where
            F: FnMut(&K, Option<E>) -> Result<T, E>,
        {
            attempt(&*self.curr, None).or_else(|e| attempt(&*self.last, Some(e)))
        }
    }
}
pub use rotating::KeyStore as RotatingKeyStore;

/// A convenience
pub type RotatingKeyFixture<A> = Arc<RwLock<Arc<RotatingKeyStore<A>>>>;
/// A trait for pointers to things that can be auto generated from itself, but only one copy should
/// exist at a time.
pub trait Generational {
    /// The error if anything goes wrong when swapping the current for the next or when attempting
    /// to access the current version.
    type Error;
    /// The data being stored.
    type Datum;
    /// Swaps out the current for the next version.
    fn advance_generation(&self) -> Result<&Self, Self::Error>;
    /// Gets the current version.
    fn get_store(&self) -> Result<Self::Datum, Self::Error>;
}
impl<K: SafeGenerateKey + Clone + Send + Sync, A: Algo<Key = K> + Send + Sync + 'static>
    Generational for RotatingKeyFixture<A>
{
    type Error = Arc<RotatingKeyStore<A>>;
    type Datum = Arc<RotatingKeyStore<A>>;
    fn advance_generation(&self) -> Result<&Self, Self::Error> {
        let mut key_store = self
            .write()
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

/// Manages and rotates keys.
///
/// The internal cleanup function must be called prior to being dropped.
#[must_use]
pub struct KeyRotator<A: Algo> {
    /// Allows access to the key store.
    pub key_store: RotatingKeyFixture<A>,
    /// Internal handle to the thread doing these rotations.
    kill_handle: Option<(Sender<()>, thread::JoinHandle<()>)>,
}

impl<K: SafeGenerateKey + Clone + Send + Sync, A: Algo<Key = K> + Send + Sync + 'static>
    KeyRotator<A>
{
    /// Initializes the key rotation mechanism.
    pub fn init(alg: A, period_between_rotation: Option<Duration>) -> Self {
        let local_copy = Arc::new(RwLock::new(Arc::new(RotatingKeyStore::new(alg))));
        let remote_copy = Arc::clone(&local_copy);

        let (tx, rx) = channel();
        const TWO_HOURS_IN_SECONDS: u64 = 2 * 60 * 60;
        let period_between_rotation =
            period_between_rotation.unwrap_or(Duration::from_secs(TWO_HOURS_IN_SECONDS));

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
                    _ => (), // continue if nothing
                }
            }
            ()
        });

        Self {
            key_store: local_copy,
            kill_handle: Some((tx, handle)),
        }
    }
    /// Cleans up the key rotation. If not called before drop, will cause a panic.
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
    /// Gets a pointer to the pointer to the stored keys.
    pub fn get_key_fixture(&self) -> RotatingKeyFixture<A> {
        Arc::clone(&self.key_store)
    }
}
// TODO isolate Rocket compatability in a feature flag.
impl<A: Algo> KeyRotator<A> {
    /// Gets a pointer to the key store that should be held by [`Rocket`](rocket::Rocket).
    pub fn get_rocket_managed_state(&self) -> RotatingKeyFixture<A> {
        Arc::clone(&self.key_store)
    }
}
