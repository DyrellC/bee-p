use crate::messages::message::Message;

use std::ops::Range;

const _TRANSACTION_BROADCAST_TYPE_ID: u8 = 4;
const TRANSACTION_BROADCAST_VARIABLE_MIN_SIZE: usize = 292;
const TRANSACTION_BROADCAST_VARIABLE_MAX_SIZE: usize = 1604;

pub struct TransactionBroadcast {
    transaction: Vec<u8>,
}

impl TransactionBroadcast {
    pub fn new(transaction: Vec<u8>) -> Self {
        Self {
            transaction: transaction,
        }
    }
}

impl Message for TransactionBroadcast {
    fn size_range() -> Range<usize> {
        (TRANSACTION_BROADCAST_VARIABLE_MIN_SIZE)..(TRANSACTION_BROADCAST_VARIABLE_MAX_SIZE + 1)
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self {
            transaction: Vec::new(),
        }
    }

    fn to_bytes(self) -> Vec<u8> {
        [].to_vec()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn empty() {}
}
