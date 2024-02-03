use core::borrow::Borrow;
use core::borrow::BorrowMut;
use std::mem::size_of;

use valida_derive::AlignedBorrow;

use crate::air::Array;
use crate::memory::MemoryReadCols;
use crate::memory::MemoryWriteCols;
use crate::utils::ec::NUM_WORDS_FIELD_ELEMENT;

pub const NUM_POSEIDON2_EXTERNAL_COLS: usize = size_of::<Poseidon2ExternalCols<u8>>();
pub const POSEIDON2_DEFAULT_ROUNDS_F: usize = 8;
pub const _POSEIDON2_DEFAULT_ROUNDS_P: usize = 22;
pub const POSEIDON2_DEFAULT_EXTERNAL_ROUNDS: usize = POSEIDON2_DEFAULT_ROUNDS_F / 2;

// It's necessary to split the struct into two parts because of the const generic parameter.
// AlignedBorrow doesn't like a struct with more than one const generic parameter.
#[derive(AlignedBorrow, Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Poseidon2ExternalCols<T>(
    pub Poseidon2ExternalColsConfigurable<T, NUM_WORDS_FIELD_ELEMENT>,
);

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Poseidon2ExternalColsConfigurable<T, const NUM_WORDS_STATE: usize> {
    pub segment: T,
    pub clk: T,

    /// An array whose i-th element records when we read the i-th word of the state.
    /// TODO: I should be able to calculate that without using this.
    pub mem_read_clk: Array<T, NUM_WORDS_STATE>,

    /// An array whose i-th element records when we write the i-th word of the state.
    /// TODO: I should be able to calculate that without using this.
    pub mem_write_clk: Array<T, NUM_WORDS_STATE>,

    pub state_ptr: T,

    pub mem_reads: Array<MemoryReadCols<T>, NUM_WORDS_STATE>,
    pub mem_writes: Array<MemoryWriteCols<T>, NUM_WORDS_STATE>,
    pub mem_addr: Array<T, NUM_WORDS_STATE>,

    pub is_external: T,

    pub is_real: T,
}
