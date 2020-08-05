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
use std::collections::HashMap;

use crate::{milestone::MilestoneIndex, protocol::Protocol, tangle::tangle};

use bee_common::worker::Error as WorkerError;

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    select,
    stream::StreamExt,
    SinkExt,
};

use log::{info, warn};

const MILESTONE_REQUEST_RANGE: u32 = 5;

pub(crate) enum MilestoneSolidifierWorkerEvent {
    Reassign(MilestoneIndex),
    Idle,
}

pub(crate) struct MilestoneSolidifierWorker {
    index: MilestoneIndex,
}

impl MilestoneSolidifierWorker {
    pub(crate) fn new(index: u32) -> Self {
        Self {
            index: MilestoneIndex(index),
        }
    }

    // async fn solidify(&self, hash: Hash, target_index: u32) -> bool {
    //     let mut missing_hashes = HashSet::new();
    //
    //     tangle().walk_approvees_depth_first(
    //         hash,
    //         |_| {},
    //         |transaction| true,
    //         |missing_hash| {
    //             missing_hashes.insert(*missing_hash);
    //         },
    //     );
    //
    //     // TODO refactor with async closures when stabilized
    //     match missing_hashes.is_empty() {
    //         true => true,
    //         false => {
    //             for missing_hash in missing_hashes {
    //                 Protocol::request_transaction(missing_hash, target_index).await;
    //             }
    //
    //             false
    //         }
    //     }
    // }
    //
    // async fn process_target(&self, target_index: u32) -> bool {
    //     match tangle().get_milestone_hash(target_index.into()) {
    //         Some(target_hash) => match self.solidify(target_hash, target_index).await {
    //             true => {
    //                 tangle().update_solid_milestone_index(target_index.into());
    //                 Protocol::broadcast_heartbeat(
    //                     *tangle().get_last_solid_milestone_index(),
    //                     *tangle().get_snapshot_milestone_index(),
    //                 )
    //                 .await;
    //                 true
    //             }
    //             false => false,
    //         },
    //         None => {
    //             // There is a gap, request the milestone
    //             Protocol::request_milestone(target_index, None);
    //             false
    //         }
    //     }
    // }

    fn request_milestones(&self) {
        // TODO this may request unpublished milestones
        if !tangle().contains_milestone(self.index) {
            Protocol::request_milestone(self.index, None);
        }
    }

    async fn solidify_milestone(&self) {
        // if let Some(target_hash) = tangle().get_milestone_hash(target_index) {
        //     if tangle().is_solid_transaction(&target_hash) {
        //         // TODO set confirmation index + trigger ledger
        //         tangle().update_last_solid_milestone_index(target_index);
        //         Protocol::broadcast_heartbeat(
        //             tangle().get_last_solid_milestone_index(),
        //             tangle().get_snapshot_milestone_index(),
        //         )
        //         .await;
        //     } else {
        //         Protocol::trigger_transaction_solidification(target_hash, target_index).await;
        //     }
        // }
        if let Some(target_hash) = tangle().get_milestone_hash(self.index) {
            if !tangle().is_solid_transaction(&target_hash) {
                Protocol::trigger_transaction_solidification(target_hash, self.index).await;
            } else {
                warn!("Milestone {} is already solid", *self.index);
                Protocol::trigger_milestone_solidification(MilestoneSolidifierCoordinatorEvent::NewSolidMilestone(
                    self.index,
                ))
                .await;
            }
        } else {
            warn!("Cannot solidify missing milestone {}", *self.index);
        }
    }

    pub(crate) async fn run(
        mut self,
        receiver: mpsc::Receiver<MilestoneSolidifierWorkerEvent>,
        shutdown: oneshot::Receiver<()>,
    ) -> Result<(), WorkerError> {
        info!("Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                _ = shutdown_fused => break,
                event = receiver_fused.next() => {
                    if let Some(event) = event {
                        match event {
                            MilestoneSolidifierWorkerEvent::Reassign(index) => {
                                self.index = index;
                                self.request_milestones();
                                self.solidify_milestone().await;
                            }
                            MilestoneSolidifierWorkerEvent::Idle => {
                                self.request_milestones();
                                self.solidify_milestone().await;
                            }
                        }
                        // while tangle().get_last_solid_milestone_index() < tangle().get_last_milestone_index() {
                        //     if !self.process_target(*tangle().get_last_solid_milestone_index() + 1).await {
                        //         break;
                        //     }
                        // }
                    }
                }
            }
        }

        info!("Stopped.");

        Ok(())
    }
}

pub enum MilestoneSolidifierCoordinatorEvent {
    NewSolidMilestone(MilestoneIndex),
    Idle,
}

pub(crate) struct MilestoneSolidifierCoordinator {
    senders: HashMap<MilestoneIndex, mpsc::Sender<MilestoneSolidifierWorkerEvent>>,
}

impl MilestoneSolidifierCoordinator {
    pub(crate) fn new(senders: HashMap<MilestoneIndex, mpsc::Sender<MilestoneSolidifierWorkerEvent>>) -> Self {
        Self { senders }
    }

    pub(crate) async fn run(
        mut self,
        receiver: mpsc::Receiver<MilestoneSolidifierCoordinatorEvent>,
        shutdown: oneshot::Receiver<()>,
    ) -> Result<(), WorkerError> {
        info!("Running.");

        let mut receiver_fused = receiver.fuse();
        let mut shutdown_fused = shutdown.fuse();

        loop {
            select! {
                _ = shutdown_fused => break,
                event = receiver_fused.next() => {
                    if let Some(event) = event {
                        match event {
                            MilestoneSolidifierCoordinatorEvent::NewSolidMilestone(index) => {
                                self.reassign_worker(index).await;
                            }
                            MilestoneSolidifierCoordinatorEvent::Idle => {
                                for sender in self.senders.values_mut() {
                                    sender.send(MilestoneSolidifierWorkerEvent::Idle).await;
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Stopped.");

        Ok(())
    }

    async fn reassign_worker(&mut self, index: MilestoneIndex) {
        let max_index = *self.senders.keys().max().unwrap() + MilestoneIndex(1);
        info!("Reasigining worker {} to {}", *index, *max_index);
        let mut sender = self.senders.remove(&index).unwrap();

        sender.send(MilestoneSolidifierWorkerEvent::Reassign(max_index)).await;
        self.senders.insert(max_index, sender);
    }
}

#[cfg(test)]
mod tests {}
