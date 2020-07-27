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

use crate::ternary::{
    wots::{Error as WotsError, WotsPrivateKey, WotsSecurityLevel},
    PrivateKeyGenerator, SIGNATURE_FRAGMENT_LENGTH,
};

use bee_crypto::ternary::{sponge::Sponge, HASH_LENGTH};
use bee_ternary::{T1B1Buf, TritBuf, Trits, T1B1};

use std::marker::PhantomData;

/// Sponge-based Winternitz One Time Signature private key generator builder.
#[derive(Default)]
pub struct WotsSpongePrivateKeyGeneratorBuilder<S> {
    security_level: Option<WotsSecurityLevel>,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> WotsSpongePrivateKeyGeneratorBuilder<S> {
    /// Sets the security level of the private key.
    pub fn security_level(mut self, security_level: WotsSecurityLevel) -> Self {
        self.security_level.replace(security_level);
        self
    }

    /// Builds the private key generator.
    pub fn build(self) -> Result<WotsSpongePrivateKeyGenerator<S>, WotsError> {
        Ok(WotsSpongePrivateKeyGenerator {
            security_level: self.security_level.ok_or(WotsError::MissingSecurityLevel)?,
            _sponge: PhantomData,
        })
    }
}

/// Sponge-based Winternitz One Time Signature private key generator.
pub struct WotsSpongePrivateKeyGenerator<S> {
    security_level: WotsSecurityLevel,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> PrivateKeyGenerator for WotsSpongePrivateKeyGenerator<S> {
    type PrivateKey = WotsPrivateKey<S>;
    type Error = WotsError;

    fn generate_from_entropy(&self, entropy: &Trits<T1B1>) -> Result<Self::PrivateKey, Self::Error> {
        if entropy.len() != HASH_LENGTH {
            return Err(WotsError::InvalidEntropyLength(entropy.len()));
        }

        let mut sponge = S::default();
        let mut state = TritBuf::<T1B1Buf>::zeros(self.security_level as usize * SIGNATURE_FRAGMENT_LENGTH);

        sponge
            .digest_into(entropy, &mut state)
            .map_err(|_| Self::Error::FailedSpongeOperation)?;

        Ok(Self::PrivateKey {
            state,
            sponge: PhantomData,
        })
    }
}
