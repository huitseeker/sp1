use std::borrow::BorrowMut;

use alloc::vec::Vec;

use p3_field::PrimeField32;
use p3_keccak_air::{generate_trace_rows, NUM_KECCAK_COLS, NUM_ROUNDS};
use p3_matrix::dense::RowMajorMatrix;

use crate::{
    air::MachineAir, runtime::ExecutionRecord, syscall::precompiles::keccak256::STATE_SIZE,
};

use super::{
    columns::{KeccakMemCols, NUM_KECCAK_MEM_COLS},
    KeccakPermuteChip,
};

impl<F: PrimeField32> MachineAir<F> for KeccakPermuteChip {
    fn name(&self) -> String {
        "KeccakPermute".to_string()
    }

    fn generate_trace(
        &self,
        input: &ExecutionRecord,
        output: &mut ExecutionRecord,
    ) -> RowMajorMatrix<F> {
        // Figure out number of total rows.
        let mut num_rows = (input.keccak_permute_events.len() * NUM_ROUNDS).next_power_of_two();
        if num_rows < 4 {
            num_rows = 4;
        }
        let mut num_total_permutations = num_rows / NUM_ROUNDS;
        if num_rows % NUM_ROUNDS != 0 {
            num_total_permutations += 1;
        }
        let num_real_permutations = input.keccak_permute_events.len();
        if num_total_permutations == 0 {
            num_total_permutations = 1;
        }

        let mut new_field_events = Vec::new();
        let mut rows = Vec::new();
        for permutation_num in 0..num_total_permutations {
            let is_real_permutation = permutation_num < num_real_permutations;

            let event = if is_real_permutation {
                Some(&input.keccak_permute_events[permutation_num])
            } else {
                None
            };

            let perm_input: [u64; STATE_SIZE] = if is_real_permutation {
                event.unwrap().pre_state
            } else {
                [0; STATE_SIZE]
            };

            let start_clk = if is_real_permutation {
                event.unwrap().clk
            } else {
                0
            };

            let shard = if is_real_permutation {
                event.unwrap().shard
            } else {
                0
            };

            // First get the trace for the plonky3 keccak air.
            let p3_keccak_trace = generate_trace_rows::<F>(vec![perm_input]);

            // Create all the rows for the permutation.
            for (i, p3_keccak_row) in (0..NUM_ROUNDS).zip(p3_keccak_trace.rows()) {
                let mut row = [F::zero(); NUM_KECCAK_COLS + NUM_KECCAK_MEM_COLS];

                // Copy the keccack row into the trace_row
                row[..NUM_KECCAK_COLS].copy_from_slice(p3_keccak_row);

                let mem_row = &mut row[NUM_KECCAK_COLS..];

                let col: &mut KeccakMemCols<F> = mem_row.borrow_mut();
                col.shard = F::from_canonical_u32(shard);
                col.clk = F::from_canonical_u32(start_clk + i as u32 * 4);

                // if this is the first row, then populate read memory accesses
                if i == 0 && is_real_permutation {
                    for (j, read_record) in event.unwrap().state_read_records.iter().enumerate() {
                        col.state_mem[j].populate_read(*read_record, &mut new_field_events);
                    }

                    col.state_addr = F::from_canonical_u32(event.unwrap().state_addr);
                    col.do_memory_check = F::one();
                }

                // if this is the last row, then populate write memory accesses
                let last_row_num = NUM_ROUNDS - 1;
                if i == last_row_num && is_real_permutation {
                    for (j, write_record) in event.unwrap().state_write_records.iter().enumerate() {
                        col.state_mem[j].populate_write(*write_record, &mut new_field_events);
                    }

                    col.state_addr = F::from_canonical_u32(event.unwrap().state_addr);
                    col.do_memory_check = F::one();
                }

                col.is_real = F::from_bool(is_real_permutation);

                rows.push(row);

                if rows.len() == num_rows {
                    break;
                }
            }
        }

        output.add_field_events(&new_field_events);

        // Convert the trace to a row major matrix.
        RowMajorMatrix::new(
            rows.into_iter().flatten().collect::<Vec<_>>(),
            NUM_KECCAK_COLS + NUM_KECCAK_MEM_COLS,
        )
    }
}
