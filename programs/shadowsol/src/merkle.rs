use anchor_lang::prelude::*;
use solana_poseidon::{hashv, Endianness, Parameters};
use crate::{ShadowState, ROOT_HISTORY_SIZE};


pub fn hash_left_right(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let hash = hashv(
        Parameters::Bn254X5, 
        Endianness::BigEndian, 
        &[&left, &right],
    ).map_err(|_| ErrorCode::PoseidonHashFailed).unwrap();
    hash.to_bytes()
}

pub fn insert_leaf(state: &mut ShadowState, leaf: [u8; 32]) -> Result<()> {
    require!(state.next_index < (1 << state.levels), ErrorCode::MerkleTreeFull);

    let mut current_index = state.next_index;
    let mut current_level_hash = leaf;
    let levels = state.levels;

    for i in 0..levels {
        let (left, right) = if current_index % 2 == 0 {
            state.filled_subtrees[i as usize] = current_level_hash;
            (current_level_hash, zeros(i))
        } else {
            (state.filled_subtrees[i as usize], current_level_hash)
        };
        current_level_hash = hash_left_right(left, right);
        current_index /= 2;
    }

    let new_root_index = (state.current_root_index + 1) % ROOT_HISTORY_SIZE;
    state.current_root_index = new_root_index;
    state.roots[new_root_index as usize] = current_level_hash;
    state.next_index += 1;
    Ok(())
}


pub fn is_known_root(state: &ShadowState, root: [u8; 32]) -> bool {
    if root == [0u8; 32] {
        return false;
    }
    let mut i = state.current_root_index;
    loop {
        if state.roots[i as usize] == root {
            return true;
        }
        if i == 0 {
            i = ROOT_HISTORY_SIZE;
        }
        i -= 1;
        if i == state.current_root_index {
            break;
        }
    }
    false
}


// pub fn get_last_root(ctx: anchor_lang::prelude::Context<crate::GetLastRoot>) -> Result<[u8; 32]> {
//     let state = &ctx.accounts.shadow_state;
//     Ok(state.roots[state.current_root_index as usize])
// }


pub fn zeros(i: u32) -> [u8; 32] {
    match i {
        0 => hex_literal::hex!("2a09a9fd93c590c26b91effbb2499f07e8f7aa12e2b4940a3aed2411cb65e11c"),
        1 => hex_literal::hex!("17192e62a157556849d93b3c6be1e2bd1f3f1660d10dd9b1ffc429aa9021252c"),
        _ => panic!("Index out of bounds"),
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Merkle tree is full. No more leaves can be added.")]
    MerkleTreeFull,
    #[msg("Poseidon hash failed")]
    PoseidonHashFailed,
}

