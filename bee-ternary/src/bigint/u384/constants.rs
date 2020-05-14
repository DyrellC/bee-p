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

use lazy_static::lazy_static;
use std::convert::TryFrom;

use crate::{
    bigint::{
        common::{BigEndian, LittleEndian, U32Repr, U8Repr},
        T242, T243,
    },
    Utrit,
};

use super::U384;

lazy_static! {
    pub static ref BE_U32_HALF_MAX: U384<BigEndian, U32Repr> = { (*LE_U32_HALF_MAX).into() };
    pub static ref BE_U32_HALF_MAX_T242: U384<BigEndian, U32Repr> =
        { Into::<U384<BigEndian, U32Repr>>::into(*LE_U32_HALF_MAX_T242) };
    pub static ref LE_U32_HALF_MAX: U384<LittleEndian, U32Repr> = {
        let mut u384_max = U384::<LittleEndian, U32Repr>::max();
        u384_max.divide_by_two();
        u384_max
    };
    pub static ref LE_U32_HALF_MAX_T242: U384<LittleEndian, U32Repr> = {
        let ut242 = T242::half_max();
        U384::<LittleEndian, U32Repr>::from(ut242)
    };
    pub static ref LE_U32_NEG_HALF_MAX_T242: U384<LittleEndian, U32Repr> = {
        let mut bigint = U384::<LittleEndian, U32Repr>::zero();
        bigint.sub_inplace(*LE_U32_HALF_MAX_T242);
        bigint
    };
    pub static ref LE_U32_ONLY_T243_OCCUPIED: U384<LittleEndian, U32Repr> = {
        let mut t243 = T243::<Utrit>::zero();
        t243.inner_mut().set(242, Utrit::One);
        U384::<LittleEndian, U32Repr>::try_from(t243).unwrap()
    };
    pub static ref LE_U32_MAX_T242: U384<LittleEndian, U32Repr> = {
        let t242 = T242::<Utrit>::max();
        U384::<LittleEndian, U32Repr>::from(t242)
    };
}

pub const BE_U8_0: U384<BigEndian, U8Repr> = U384::<BigEndian, U8Repr>::from_array([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

pub const BE_U8_1: U384<BigEndian, U8Repr> = U384::<BigEndian, U8Repr>::from_array([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
]);

pub const BE_U8_MAX: U384<BigEndian, U8Repr> = U384::<BigEndian, U8Repr>::from_array([
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
]);

pub const BE_U8_2: U384<BigEndian, U8Repr> = U384::<BigEndian, U8Repr>::from_array([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
]);

pub const BE_U32_0: U384<BigEndian, U32Repr> = U384::<BigEndian, U32Repr>::from_array([
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
]);

pub const BE_U32_1: U384<BigEndian, U32Repr> = U384::<BigEndian, U32Repr>::from_array([
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0001,
]);

pub const BE_U32_MAX: U384<BigEndian, U32Repr> = U384::<BigEndian, U32Repr>::from_array([
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
]);

pub const BE_U32_2: U384<BigEndian, U32Repr> = U384::<BigEndian, U32Repr>::from_array([
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0002,
]);

pub const LE_U8_0: U384<LittleEndian, U8Repr> = U384::<LittleEndian, U8Repr>::from_array([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

pub const LE_U8_1: U384<LittleEndian, U8Repr> = U384::<LittleEndian, U8Repr>::from_array([
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

pub const LE_U8_MAX: U384<LittleEndian, U8Repr> = U384::<LittleEndian, U8Repr>::from_array([
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
]);

pub const LE_U8_2: U384<LittleEndian, U8Repr> = U384::<LittleEndian, U8Repr>::from_array([
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

pub const LE_U32_0: U384<LittleEndian, U32Repr> = U384::<LittleEndian, U32Repr>::from_array([
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
]);

pub const LE_U32_1: U384<LittleEndian, U32Repr> = U384::<LittleEndian, U32Repr>::from_array([
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
]);

pub const LE_U32_2: U384<LittleEndian, U32Repr> = U384::<LittleEndian, U32Repr>::from_array([
    0x0000_0002,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
]);

pub const LE_U32_MAX: U384<LittleEndian, U32Repr> = U384::<LittleEndian, U32Repr>::from_array([
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
    0xffff_ffff,
]);
