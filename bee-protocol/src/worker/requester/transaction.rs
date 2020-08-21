// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{
    message::TransactionRequest, milestone::MilestoneIndex, protocol::Protocol, worker::SenderWorker,
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::wait_priority_queue::WaitIncoming;
use bee_crypto::ternary::Hash;
use bee_ternary::T5B1Buf;

use async_std::stream::{interval, Interval};
use bytemuck::cast_slice;
use futures::{select, stream::Fuse, FutureExt, StreamExt};
use log::info;

use std::{
    cmp::Ordering,
    time::{Duration, Instant},
};

const RETRY_INTERVAL_SECS: u64 = 5;

type Receiver<'a> = ShutdownStream<WaitIncoming<'a, TransactionRequesterWorkerEntry>>;

#[derive(Eq, PartialEq)]
pub(crate) struct TransactionRequesterWorkerEntry(pub(crate) Hash, pub(crate) MilestoneIndex);

impl PartialOrd for TransactionRequesterWorkerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.1.partial_cmp(&self.1)
    }
}

impl Ord for TransactionRequesterWorkerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.1.cmp(&self.1)
    }
}

pub(crate) struct TransactionRequesterWorker<'a> {
    counter: usize,
    receiver: Receiver<'a>,
    timeouts: Fuse<Interval>,
}

impl<'a> TransactionRequesterWorker<'a> {
    pub(crate) fn new(receiver: Receiver<'a>) -> Self {
        Self {
            counter: 0,
            receiver,
            timeouts: interval(Duration::from_secs(RETRY_INTERVAL_SECS)).fuse(),
        }
    }

    async fn process_request(&mut self, hash: Hash, index: MilestoneIndex) {
        if Protocol::get().requested_transactions.contains_key(&hash) {
            return;
        }

        self.process_request_unchecked(hash, index).await;
    }

    async fn process_request_unchecked(&mut self, hash: Hash, index: MilestoneIndex) {
        if Protocol::get().peer_manager.handshaked_peers.is_empty() {
            return;
        }

        let guard = Protocol::get().peer_manager.handshaked_peers_keys.read().await;

        for _ in 0..guard.len() {
            let epid = &guard[self.counter % guard.len()];

            self.counter += 1;

            if let Some(peer) = Protocol::get().peer_manager.handshaked_peers.get(epid) {
                if index > peer.snapshot_milestone_index() && index <= peer.last_solid_milestone_index() {
                    Protocol::get()
                        .requested_transactions
                        .insert(hash, (index, Instant::now()));
                    SenderWorker::<TransactionRequest>::send(
                        epid,
                        TransactionRequest::new(cast_slice(hash.as_trits().encode::<T5B1Buf>().as_i8_slice())),
                    );
                    break;
                }
            }
        }
    }

    async fn retry_requests(&mut self) {
        let mut retry_counts = 0;
        for mut tx in Protocol::get().requested_transactions.iter_mut() {
            let (hash, (index, instant)) = tx.pair_mut();
            let now = Instant::now();
            if (now - *instant).as_secs() > RETRY_INTERVAL_SECS {
                *instant = now;
                self.process_request_unchecked(hash.clone(), *index).await;
                retry_counts += 1;
            }
        }
        info!("Retried {} transactions", retry_counts);
    }

    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        info!("Running.");

        loop {
            select! {
                // FIXME: Receiver is already fused
                entry = self.receiver.next().fuse() => match entry {
                    Some(TransactionRequesterWorkerEntry(hash, index)) => self.process_request(hash, index).await,
                    None => break,
                },
                _ = self.timeouts.next() => self.retry_requests().await,
            }
        }

        info!("Stopped.");

        Ok(())
    }
}
