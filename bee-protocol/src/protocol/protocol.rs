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
    config::ProtocolConfig,
    events::{LastMilestone, LastSolidMilestone},
    milestone::MilestoneIndex,
    peer::{Peer, PeerManager},
    protocol::ProtocolMetrics,
    tangle::tangle,
    worker::{
        BroadcasterWorker, BroadcasterWorkerEvent, MilestoneRequesterWorker, MilestoneRequesterWorkerEntry,
        MilestoneResponderWorker, MilestoneResponderWorkerEvent, MilestoneSolidifierWorker,
        MilestoneSolidifierWorkerEvent, MilestoneValidatorWorker, MilestoneValidatorWorkerEvent, PeerHandshakerWorker,
        StatusWorker, TpsWorker, TransactionRequesterWorker, TransactionRequesterWorkerEntry,
        TransactionResponderWorker, TransactionResponderWorkerEvent, TransactionSolidifierWorker,
        TransactionSolidifierWorkerEvent, TransactionWorker, TransactionWorkerEvent,
    },
};

use bee_common::shutdown::Shutdown;
use bee_common_ext::{wait_priority_queue::WaitPriorityQueue, event::Bus};
use bee_crypto::ternary::{
    sponge::{CurlP27, CurlP81, Kerl, SpongeKind},
    Hash,
};
use bee_network::{Address, EndpointId, Network, Origin};
use bee_signing::ternary::wots::WotsPublicKey;

use std::{ptr, sync::Arc};

use async_std::task::{block_on, spawn};
use dashmap::DashMap;
use futures::channel::{mpsc, oneshot};
use log::{info, warn};

static mut PROTOCOL: *const Protocol = ptr::null();

pub struct Protocol {
    pub(crate) config: ProtocolConfig,
    pub(crate) network: Network,
    pub(crate) bus: Arc<Bus<'static>>,
    pub(crate) metrics: ProtocolMetrics,
    pub(crate) transaction_worker: mpsc::Sender<TransactionWorkerEvent>,
    pub(crate) transaction_responder_worker: mpsc::Sender<TransactionResponderWorkerEvent>,
    pub(crate) milestone_responder_worker: mpsc::Sender<MilestoneResponderWorkerEvent>,
    pub(crate) transaction_requester_worker: WaitPriorityQueue<TransactionRequesterWorkerEntry>,
    pub(crate) milestone_requester_worker: WaitPriorityQueue<MilestoneRequesterWorkerEntry>,
    pub(crate) milestone_validator_worker: mpsc::Sender<MilestoneValidatorWorkerEvent>,
    pub(crate) transaction_solidifier_worker: mpsc::Sender<TransactionSolidifierWorkerEvent>,
    pub(crate) milestone_solidifier_worker: mpsc::Sender<MilestoneSolidifierWorkerEvent>,
    pub(crate) broadcaster_worker: mpsc::Sender<BroadcasterWorkerEvent>,
    pub(crate) peer_manager: PeerManager,
    pub(crate) requested: DashMap<Hash, MilestoneIndex>,
}

impl Protocol {
    pub async fn init(config: ProtocolConfig, network: Network, bus: Arc<Bus<'static>>, shutdown: &mut Shutdown) {
        if unsafe { !PROTOCOL.is_null() } {
            warn!("Already initialized.");
            return;
        }

        let (transaction_worker_tx, transaction_worker_rx) = mpsc::channel(config.workers.transaction_worker_bound);
        let (transaction_worker_shutdown_tx, transaction_worker_shutdown_rx) = oneshot::channel();

        let (transaction_responder_worker_tx, transaction_responder_worker_rx) =
            mpsc::channel(config.workers.transaction_responder_worker_bound);
        let (transaction_responder_worker_shutdown_tx, transaction_responder_worker_shutdown_rx) = oneshot::channel();

        let (milestone_responder_worker_tx, milestone_responder_worker_rx) =
            mpsc::channel(config.workers.milestone_responder_worker_bound);
        let (milestone_responder_worker_shutdown_tx, milestone_responder_worker_shutdown_rx) = oneshot::channel();

        let (transaction_requester_worker_shutdown_tx, transaction_requester_worker_shutdown_rx) = oneshot::channel();

        let (milestone_requester_worker_shutdown_tx, milestone_requester_worker_shutdown_rx) = oneshot::channel();

        let (milestone_validator_worker_tx, milestone_validator_worker_rx) =
            mpsc::channel(config.workers.milestone_validator_worker_bound);
        let (milestone_validator_worker_shutdown_tx, milestone_validator_worker_shutdown_rx) = oneshot::channel();

        let (transaction_solidifier_worker_tx, transaction_solidifier_worker_rx) =
            mpsc::channel(config.workers.transaction_solidifier_worker_bound);
        let (transaction_solidifier_worker_shutdown_tx, transaction_solidifier_worker_shutdown_rx) = oneshot::channel();

        let (milestone_solidifier_worker_tx, milestone_solidifier_worker_rx) =
            mpsc::channel(config.workers.milestone_solidifier_worker_bound);
        let (milestone_solidifier_worker_shutdown_tx, milestone_solidifier_worker_shutdown_rx) = oneshot::channel();

        let (broadcaster_worker_tx, broadcaster_worker_rx) = mpsc::channel(config.workers.broadcaster_worker_bound);
        let (broadcaster_worker_shutdown_tx, broadcaster_worker_shutdown_rx) = oneshot::channel();

        let (status_worker_shutdown_tx, status_worker_shutdown_rx) = oneshot::channel();

        let (tps_worker_shutdown_tx, tps_worker_shutdown_rx) = oneshot::channel();

        let protocol = Protocol {
            config,
            network: network.clone(),
            bus,
            metrics: ProtocolMetrics::new(),
            transaction_worker: transaction_worker_tx,
            transaction_responder_worker: transaction_responder_worker_tx,
            milestone_responder_worker: milestone_responder_worker_tx,
            transaction_requester_worker: Default::default(),
            milestone_requester_worker: Default::default(),
            milestone_validator_worker: milestone_validator_worker_tx,
            transaction_solidifier_worker: transaction_solidifier_worker_tx,
            milestone_solidifier_worker: milestone_solidifier_worker_tx,
            broadcaster_worker: broadcaster_worker_tx,
            peer_manager: PeerManager::new(network.clone()),
            requested: Default::default(),
        };

        unsafe {
            PROTOCOL = Box::leak(protocol.into()) as *const _;
        }

        Protocol::get().bus.add_listener(handle_last_milestone);
        Protocol::get().bus.add_listener(handle_last_solid_milestone);

        shutdown.add_worker_shutdown(
            transaction_worker_shutdown_tx,
            spawn(
                TransactionWorker::new(
                    Protocol::get().milestone_validator_worker.clone(),
                    Protocol::get().config.workers.transaction_worker_cache,
                )
                .run(transaction_worker_rx, transaction_worker_shutdown_rx),
            ),
        );

        shutdown.add_worker_shutdown(
            transaction_responder_worker_shutdown_tx,
            spawn(TransactionResponderWorker::new().run(
                transaction_responder_worker_rx,
                transaction_responder_worker_shutdown_rx,
            )),
        );

        shutdown.add_worker_shutdown(
            milestone_responder_worker_shutdown_tx,
            spawn(
                MilestoneResponderWorker::new()
                    .run(milestone_responder_worker_rx, milestone_responder_worker_shutdown_rx),
            ),
        );

        shutdown.add_worker_shutdown(
            transaction_requester_worker_shutdown_tx,
            spawn(TransactionRequesterWorker::new().run(transaction_requester_worker_shutdown_rx)),
        );

        shutdown.add_worker_shutdown(
            milestone_requester_worker_shutdown_tx,
            spawn(MilestoneRequesterWorker::new().run(milestone_requester_worker_shutdown_rx)),
        );

        match Protocol::get().config.coordinator.sponge_type {
            SpongeKind::Kerl => shutdown.add_worker_shutdown(
                milestone_validator_worker_shutdown_tx,
                spawn(
                    MilestoneValidatorWorker::<Kerl, WotsPublicKey<Kerl>>::new()
                        .run(milestone_validator_worker_rx, milestone_validator_worker_shutdown_rx),
                ),
            ),
            SpongeKind::CurlP27 => shutdown.add_worker_shutdown(
                milestone_validator_worker_shutdown_tx,
                spawn(
                    MilestoneValidatorWorker::<CurlP27, WotsPublicKey<CurlP27>>::new()
                        .run(milestone_validator_worker_rx, milestone_validator_worker_shutdown_rx),
                ),
            ),
            SpongeKind::CurlP81 => shutdown.add_worker_shutdown(
                milestone_validator_worker_shutdown_tx,
                spawn(
                    MilestoneValidatorWorker::<CurlP81, WotsPublicKey<CurlP81>>::new()
                        .run(milestone_validator_worker_rx, milestone_validator_worker_shutdown_rx),
                ),
            ),
        };

        shutdown.add_worker_shutdown(
            transaction_solidifier_worker_shutdown_tx,
            spawn(TransactionSolidifierWorker::new().run(
                transaction_solidifier_worker_rx,
                transaction_solidifier_worker_shutdown_rx,
            )),
        );

        shutdown.add_worker_shutdown(
            milestone_solidifier_worker_shutdown_tx,
            spawn(
                MilestoneSolidifierWorker::new()
                    .run(milestone_solidifier_worker_rx, milestone_solidifier_worker_shutdown_rx),
            ),
        );

        shutdown.add_worker_shutdown(
            broadcaster_worker_shutdown_tx,
            spawn(BroadcasterWorker::new(network).run(broadcaster_worker_rx, broadcaster_worker_shutdown_rx)),
        );

        shutdown.add_worker_shutdown(
            status_worker_shutdown_tx,
            spawn(StatusWorker::new(Protocol::get().config.workers.status_interval).run(status_worker_shutdown_rx)),
        );

        shutdown.add_worker_shutdown(
            tps_worker_shutdown_tx,
            spawn(TpsWorker::new().run(tps_worker_shutdown_rx)),
        );
    }

    pub(crate) fn get() -> &'static Protocol {
        if unsafe { PROTOCOL.is_null() } {
            panic!("Uninitialized protocol.");
        } else {
            unsafe { &*PROTOCOL }
        }
    }

    pub fn register(
        epid: EndpointId,
        address: Address,
        origin: Origin,
    ) -> (mpsc::Sender<Vec<u8>>, oneshot::Sender<()>) {
        // TODO check if not already added ?

        let peer = Arc::new(Peer::new(epid, address, origin));

        let (receiver_tx, receiver_rx) = mpsc::channel(Protocol::get().config.workers.receiver_worker_bound);
        let (receiver_shutdown_tx, receiver_shutdown_rx) = oneshot::channel();

        Protocol::get().peer_manager.add(peer.clone());

        spawn(PeerHandshakerWorker::new(Protocol::get().network.clone(), peer).run(receiver_rx, receiver_shutdown_rx));

        (receiver_tx, receiver_shutdown_tx)
    }
}

fn handle_last_milestone(last_milestone: &LastMilestone) {
    info!("New milestone #{}.", *last_milestone.0.index);
    tangle().update_last_milestone_index(last_milestone.0.index);
}

fn handle_last_solid_milestone(last_solid_milestone: &LastSolidMilestone) {
    tangle().update_last_solid_milestone_index(last_solid_milestone.0.index);
    // TODO block_on ?
    block_on(Protocol::broadcast_heartbeat(
        last_solid_milestone.0.index,
        tangle().get_snapshot_milestone_index(),
    ));
}
