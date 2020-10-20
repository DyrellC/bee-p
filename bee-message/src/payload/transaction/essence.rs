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
    payload::{
        transaction::{
            constants::{INPUT_OUTPUT_COUNT_RANGE, INPUT_OUTPUT_INDEX_RANGE},
            input::Input,
            output::Output,
        },
        Payload,
    },
    Error,
};

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, vec::Vec};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionEssence {
    inputs: Box<[Input]>,
    outputs: Box<[Output]>,
    payload: Option<Payload>,
}

impl TransactionEssence {
    pub fn builder() -> TransactionEssenceBuilder {
        TransactionEssenceBuilder::new()
    }

    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    pub fn payload(&self) -> &Option<Payload> {
        &self.payload
    }
}

impl Packable for TransactionEssence {
    fn packed_len(&self) -> usize {
        0u8.packed_len()
            + 0u16.packed_len()
            + self.inputs.iter().map(|input| input.packed_len()).sum::<usize>()
            + 0u16.packed_len()
            + self.outputs.iter().map(|output| output.packed_len()).sum::<usize>()
            + 0u32.packed_len()
            + self.payload.iter().map(|payload| payload.packed_len()).sum::<usize>()
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        0u8.pack(buf)?;

        (self.inputs.len() as u16).pack(buf)?;
        for input in self.inputs.iter() {
            input.pack(buf)?;
        }

        (self.outputs.len() as u16).pack(buf)?;
        for output in self.outputs.iter() {
            output.pack(buf)?;
        }

        match self.payload {
            Some(ref payload) => {
                (payload.packed_len() as u32).pack(buf)?;
                payload.pack(buf)?;
            }
            None => 0u32.pack(buf)?,
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        if u8::unpack(buf)? != 0u8 {
            return Err(PackableError::InvalidType);
        }

        let inputs_len = u16::unpack(buf)? as usize;
        let mut inputs = Vec::with_capacity(inputs_len);
        for _ in 0..inputs_len {
            inputs.push(Input::unpack(buf)?);
        }

        let outputs_len = u16::unpack(buf)? as usize;
        let mut outputs = Vec::with_capacity(outputs_len);
        for _ in 0..outputs_len {
            outputs.push(Output::unpack(buf)?);
        }

        let payload_len = u32::unpack(buf)? as usize;
        let payload = if payload_len > 0 {
            let payload = Payload::unpack(buf)?;
            if payload_len != payload.packed_len() {
                return Err(PackableError::InvalidAnnouncedLen);
            }

            Some(payload)
        } else {
            None
        };

        Ok(Self {
            inputs: inputs.into_boxed_slice(),
            outputs: outputs.into_boxed_slice(),
            payload,
        })
    }
}

#[derive(Debug, Default)]
pub struct TransactionEssenceBuilder {
    pub(crate) inputs: Vec<Input>,
    pub(crate) outputs: Vec<Output>,
    pub(crate) payload: Option<Payload>,
}

impl TransactionEssenceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn add_output(mut self, output: Output) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn with_payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn finish(self) -> Result<TransactionEssence, Error> {
        // Inputs validation

        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.inputs.len()) {
            return Err(Error::CountError);
        }

        for i in self.inputs.iter() {
            // Input Type value must be 0, denoting an UTXO Input.
            match i {
                Input::UTXO(u) => {
                    // Transaction Output Index must be 0 ≤ x < 127
                    if !INPUT_OUTPUT_INDEX_RANGE.contains(&u.index()) {
                        return Err(Error::CountError);
                    }

                    // Every combination of Transaction ID + Transaction Output Index must be unique in the inputs set.
                    if self.inputs.iter().filter(|j| *j == i).count() > 1 {
                        return Err(Error::DuplicateError);
                    }
                }
            }
        }

        // Inputs must be in lexicographical order of their serialized form.
        // TODO
        // if !is_sorted(transaction.inputs.iter()) {
        //     return Err(Error::OrderError);
        // }

        // Output validation

        if !INPUT_OUTPUT_COUNT_RANGE.contains(&self.outputs.len()) {
            return Err(Error::CountError);
        }

        let mut total = 0;
        for i in self.outputs.iter() {
            // Output Type must be 0, denoting a SignatureLockedSingle.
            match i {
                Output::SignatureLockedSingle(u) => {
                    // Address Type must either be 0 or 1, denoting a WOTS- or Ed25519 address.

                    // If Address is of type WOTS address, its bytes must be valid T5B1 bytes.

                    // The Address must be unique in the set of SigLockedSingleDeposits
                    if self
                        .outputs
                        .iter()
                        .filter(|j| match *j {
                            Output::SignatureLockedSingle(s) => s.address() == u.address(),
                        })
                        .count()
                        > 1
                    {
                        return Err(Error::DuplicateError);
                    }

                    // Amount must be > 0
                    let amount = u.amount().get();
                    if amount == 0 {
                        return Err(Error::AmountError);
                    }

                    total += amount;
                }
            }
        }

        // Outputs must be in lexicographical order by their serialized form
        // TODO
        // if !is_sorted(transaction.outputs.iter()) {
        //     return Err(Error::OrderError);
        // }

        // Accumulated output balance must not exceed the total supply of tokens 2'779'530'283'277'761
        if total > 2779530283277761 {
            return Err(Error::AmountError);
        }

        // Payload Length must be 0 (to indicate that there's no payload) or be valid for the specified payload type.
        // Payload Type must be one of the supported payload types if Payload Length is not 0.

        Ok(TransactionEssence {
            inputs: self.inputs.into_boxed_slice(),
            outputs: self.outputs.into_boxed_slice(),
            payload: self.payload,
        })
    }
}