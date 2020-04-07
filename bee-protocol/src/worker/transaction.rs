use async_std::task::{
    block_on,
    spawn
};

use bee_bundle::{
    Hash,
    Transaction,
    TransactionField
};

use bee_common::constants;

use bee_crypto::{
    CurlP81,
    Sponge
};

use bee_tangle::tangle;

use bee_ternary::{
    Error,
    Trits,
    TritBuf,
    T5B1,
    T5B1Buf,
    T1B1Buf
};

use crate::message::TransactionBroadcast;

use futures::{
    channel::{
        mpsc,
        oneshot,
    },
    future::FutureExt,
    select,
    stream::StreamExt,
    prelude::*,
};

use log::info;

use std::collections::{
    HashSet,
    VecDeque
};
use std::hash::BuildHasherDefault;
use std::hash::Hasher;

use twox_hash::XxHash64;

pub(crate) type TransactionWorkerEvent = TransactionBroadcast;

pub(crate) struct TransactionWorker;

impl TransactionWorker {

    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) async fn run(self, receiver: mpsc::Receiver<TransactionWorkerEvent>, shutdown: oneshot::Receiver<()>) {

        info!("[TransactionWorker ] Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        let mut curl = CurlP81::new();
        let mut cache = TinyHashCache::new(10000);

        loop {

            let transaction_broadcast: TransactionBroadcast = select! {

                transaction_broadcast = receiver_fused.next() => match transaction_broadcast {

                    Some(transaction_broadcast) => transaction_broadcast,
                    None => {
                        info!("[TransactionWorker ] Unable to receive transactions from channel.");
                        break;
                    },

                },

                _ = shutdown_fused => break

            };

            info!("[TransactionWorker ] Processing received data...");

            if !cache.insert(xx_hash(transaction_broadcast.transaction.as_slice())) {
                info!("[TransactionWorker ] Data already received.");
                continue;
            }

            // convert received transaction bytes into T1B1 buffer
            let transaction_buf: TritBuf<T1B1Buf> = {

                let mut raw_bytes = transaction_broadcast.transaction;
                while raw_bytes.len() < constants::TRANSACTION_BYTE_LEN {
                    raw_bytes.push(0);
                }

                // transform &[u8] to &[i8]
                let t5b1_bytes: &[i8] = unsafe { &*(raw_bytes.as_slice() as *const [u8] as *const [i8]) };

                // get T5B1 trits
                let t5b1_trits_result: Result<&Trits<T5B1>, Error> = Trits::<T5B1>::try_from_raw(t5b1_bytes, t5b1_bytes.len() * 5 - 1);

                match t5b1_trits_result {
                    Ok(t5b1_trits) => {

                        // get T5B1 trit_buf
                        let t5b1_trit_buf: TritBuf<T5B1Buf> = t5b1_trits.to_buf::<T5B1Buf>();

                        // get T1B1 trit_buf from TB51 trit_buf
                        t5b1_trit_buf.encode::<T1B1Buf>()

                    },
                    Err(_) => {
                        info!("[TransactionWorker ] Can not decode T5B1 from received data.");
                        continue;
                    }
                }

            };

            if transaction_buf.len() > constants::TRANSACTION_TRIT_LEN {
                info!("[TransactionWorker ] Received buffer is bigger then allowed size.");
                continue;
            }

            // build transaction
            let transaction_result = Transaction::from_trits(&transaction_buf);

            // validate transaction result
            let built_transaction = match transaction_result {
                Ok(tx) => tx,
                Err(_) => {
                    info!("[TransactionWorker ] Can not build transaction from received data.");
                    continue;
                }
            };

            // calculate transaction hash
            let tx_hash: Hash = Hash::from_inner_unchecked(curl.digest(&transaction_buf).unwrap());

            info!("[TransactionWorker ] Received transaction {}.", &tx_hash);

            // check if transactions is already present in the tangle before doing any further work
            if  tangle().contains_transaction(&tx_hash) {
                info!("[TransactionWorker ] Transaction {} already present in the tangle.", &tx_hash);
                continue;
            }

            // store transaction
            tangle().insert_transaction(built_transaction, tx_hash).await;

        }

        info!("[TransactionWorker ] Stopped.");

    }
}

struct CustomHasher {
    result: Option<u64>,
}

impl CustomHasher {
    fn finish(&self) -> u64 {
        self.result.unwrap()
    }
    fn write(&mut self, i: u64) {
        self.result.replace(i);
    }
}

impl Default for CustomHasher {
    fn default() -> Self {
        Self { result: None }
    }
}

impl Hasher for CustomHasher {

    fn finish(&self) -> u64 {
        CustomHasher::finish(self)
    }
    fn write(&mut self, bytes: &[u8]) {
        use std::convert::TryInto;
        let (int_bytes, _rest) = bytes.split_at(std::mem::size_of::<u64>());
        let i = u64::from_ne_bytes(int_bytes.try_into().unwrap());
        CustomHasher::write(self, i);
    }

}

struct TinyHashCache {
    max_capacity: usize,
    cache: HashSet<u64, BuildHasherDefault<CustomHasher>>,
    elem_order: VecDeque<u64>,
}

impl TinyHashCache {

    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity,
            cache: HashSet::default(),
            elem_order: VecDeque::new()
        }
    }

    pub fn insert(&mut self, hash: u64) -> bool {

        if self.contains(&hash) {
            return false;
        }

        if self.cache.len() >= self.max_capacity {
            let first  = self.elem_order.pop_front().unwrap();
            self.cache.remove(&first);
        }

        self.cache.insert(hash.clone());
        self.elem_order.push_back(hash);

        true

    }

    fn contains(&self, hash: &u64) -> bool {
        self.cache.contains(hash)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

}

fn xx_hash(buf: &[u8]) -> u64 {
    let mut hasher = XxHash64::default();
    hasher.write(buf);
    hasher.finish()
}

#[test]
fn test_cache_insert_same_elements() {

    let mut cache = TinyHashCache::new(10);

    let first_buf = &[1,2,3];
    let second_buf = &[1,2,3];

    assert_eq!(cache.insert(xx_hash(first_buf)), true);
    assert_eq!(cache.insert(xx_hash(second_buf)), false);
    assert_eq!(cache.len(), 1);

}

#[test]
fn test_cache_insert_different_elements() {

    let mut cache = TinyHashCache::new(10);

    let first_buf = &[1,2,3];
    let second_buf = &[3,4,5];

    assert_eq!(cache.insert(xx_hash(first_buf)), true);
    assert_eq!(cache.insert(xx_hash(second_buf)), true);
    assert_eq!(cache.len(), 2);

}

#[test]
fn test_cache_max_capacity() {

    let mut cache = TinyHashCache::new(1);

    let first_buf = &[1,2,3];
    let second_buf = &[3,4,5];
    let second_buf_clone = second_buf.clone();

    assert_eq!(cache.insert(xx_hash(first_buf)), true);
    assert_eq!(cache.insert(xx_hash(second_buf)), true);
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.insert(xx_hash(&second_buf_clone)), false);

}

#[test]
fn test_tx_worker() {

    bee_tangle::init();

    assert_eq!(tangle().size(), 0);

    let (transaction_worker_sender, transaction_worker_receiver) = mpsc::channel(1000);
    let (mut shutdown_sender, shutdown_receiver) = oneshot::channel();

    let tx: [u8; 1024] = [0; 1024];
    assert_eq!(tx.len() <= constants::TRANSACTION_BYTE_LEN, true);
    let message = TransactionBroadcast::new(&tx);

    let mut transaction_worker_sender_clone = transaction_worker_sender.clone();
    spawn(async move {
        transaction_worker_sender_clone.send(message).await.unwrap();
    });

    spawn(async move {
        use async_std::task;
        use std::time::Duration;
        task::sleep(Duration::from_secs(1)).await;
        shutdown_sender.send(()).unwrap();
    });

    block_on(TransactionWorker::new().run(transaction_worker_receiver, shutdown_receiver));

    assert_eq!(tangle().size(), 1);
    assert_eq!(tangle().contains_transaction(&Hash::zeros()), true);

}