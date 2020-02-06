#![allow(clippy::default_trait_access)]

//temporary Lighthouse SSZ and hashing implementation
use bls::PublicKeyBytes;
use ethereum_types::H256 as Hash256;
use serde::{Deserialize, Serialize};
use ssz_new_derive::{SszDecode, SszEncode};
use ssz_types::{BitList, FixedVector, VariableList};
use tree_hash::TreeHash;
use tree_hash_derive::{SignedRoot, TreeHash};
use typenum::{Sum, U1};

use crate::config::*;
use crate::consts;
use crate::primitives::*;

#[derive(
    Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash, SignedRoot,
)]
pub struct Attestation<C: Config> {
    pub aggregation_bits: BitList<C::MaxValidatorsPerCommittee>,
    pub data: AttestationData,
    #[signed_root(skip_hashing)]
    pub signature: AggregateSignatureBytes,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Hash,
    Deserialize,
    Serialize,
    SszEncode,
    SszDecode,
    TreeHash,
    SignedRoot,
    Default,
)]
pub struct AttestationData {
    pub slot: Slot,
    pub index: u64,
    pub beacon_block_root: H256,
    pub source: Checkpoint,
    pub target: Checkpoint,
}

#[derive(
    Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash, SignedRoot,
)]
pub struct AttestationDataAndCustodyBit {
    pub data: AttestationData,
    pub custody_bit: bool,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Default,
    Deserialize,
    Serialize,
    SszEncode,
    SszDecode,
    TreeHash,
    SignedRoot,
)]
pub struct AttestationDuty {
    pub slot: Slot,
    pub shard: Shard,
    pub committee_index: usize,
    pub committee_len: usize,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct AttesterSlashing<C: Config> {
    pub attestation_1: IndexedAttestation<C>,
    pub attestation_2: IndexedAttestation<C>,
}

#[derive(
    Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash, SignedRoot,
)]
pub struct BeaconBlock<C: Config> {
    pub slot: Slot,
    pub parent_root: H256,
    pub state_root: H256,
    pub body: BeaconBlockBody<C>,
    #[signed_root(skip_hashing)]
    pub signature: SignatureBytes,
}

impl<C: Config> Default for BeaconBlock<C> {
    fn default() -> Self {
        Self {
            slot: Default::default(),
            parent_root: Default::default(),
            state_root: Default::default(),
            body: Default::default(),
            signature: SignatureBytes::empty(),
        }
    }
}

#[derive(
    Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash, SignedRoot,
)]
pub struct BeaconBlockBody<C: Config> {
    pub randao_reveal: SignatureBytes,
    pub eth1_data: Eth1Data,
    pub graffiti: H256,
    pub proposer_slashings: VariableList<ProposerSlashing, C::MaxProposerSlashings>,
    pub attester_slashings: VariableList<AttesterSlashing<C>, C::MaxAttesterSlashings>,
    pub attestations: VariableList<Attestation<C>, C::MaxAttestations>,
    pub deposits: VariableList<Deposit, C::MaxDeposits>,
    pub voluntary_exits: VariableList<VoluntaryExit, C::MaxVoluntaryExits>,
}

impl<C: Config> Default for BeaconBlockBody<C> {
    fn default() -> Self {
        Self {
            randao_reveal: SignatureBytes::empty(),
            eth1_data: Default::default(),
            graffiti: Default::default(),
            proposer_slashings: Default::default(),
            attester_slashings: Default::default(),
            attestations: Default::default(),
            deposits: Default::default(),
            voluntary_exits: Default::default(),
        }
    }
}

#[derive(
    Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash, SignedRoot,
)]
pub struct BeaconBlockHeader {
    pub slot: Slot,
    pub parent_root: H256,
    pub state_root: H256,
    pub body_root: H256,
    #[signed_root(skip_hashing)]
    pub signature: SignatureBytes,
}

impl Default for BeaconBlockHeader {
    fn default() -> Self {
        Self {
            slot: Default::default(),
            parent_root: Default::default(),
            state_root: Default::default(),
            body_root: Default::default(),
            signature: SignatureBytes::empty(),
        }
    }
}

impl BeaconBlockHeader {
    pub fn canonical_root(&self) -> Hash256 {
        Hash256::from_slice(&self.tree_hash_root()[..])
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Default,
    Hash,
    Deserialize,
    Serialize,
    SszEncode,
    SszDecode,
    TreeHash,
)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: H256,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Default,
    Hash,
    Deserialize,
    Serialize,
    SszEncode,
    SszDecode,
    TreeHash,
)]
pub struct Crosslink {
    pub shard: u64,
    pub parent_root: H256,
    pub start_epoch: Epoch,
    pub end_epoch: Epoch,
    pub data_root: H256,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct Deposit {
    pub proof: FixedVector<H256, Sum<consts::DepositContractTreeDepth, U1>>,
    pub data: DepositData,
}

#[derive(
    Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash, SignedRoot,
)]
pub struct DepositData {
    pub pubkey: PublicKeyBytes,
    pub withdrawal_credentials: H256,
    pub amount: u64,
    #[signed_root(skip_hashing)]
    pub signature: SignatureBytes,
}

#[derive(
    Clone, PartialEq, Eq, Debug, Default, Deserialize, Serialize, SszEncode, SszDecode, TreeHash,
)]
pub struct Eth1Data {
    pub deposit_root: H256,
    pub deposit_count: u64,
    pub block_hash: H256,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Deserialize,
    Serialize,
    SszEncode,
    SszDecode,
    TreeHash,
    SignedRoot,
    Default,
)]
pub struct Fork {
    pub previous_version: Version,
    pub current_version: Version,
    pub epoch: Epoch,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct HistoricalBatch<C: Config> {
    pub block_roots: FixedVector<H256, C::SlotsPerHistoricalRoot>,
    pub state_roots: FixedVector<H256, C::SlotsPerHistoricalRoot>,
}

#[derive(
    Clone,
    PartialEq,
    Debug,
    Deserialize,
    Serialize,
    SszEncode,
    SszDecode,
    TreeHash,
    SignedRoot,
    Default,
)]
pub struct IndexedAttestation<C: Config> {
    pub attesting_indices: VariableList<u64, C::MaxValidatorsPerCommittee>,
    pub data: AttestationData,
    #[signed_root(skip_hashing)]
    pub signature: AggregateSignatureBytes,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct PendingAttestation<C: Config> {
    pub aggregation_bits: BitList<C::MaxValidatorsPerCommittee>,
    pub data: AttestationData,
    pub inclusion_delay: u64,
    pub proposer_index: u64,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]

pub struct ProposerSlashing {
    pub proposer_index: u64,
    pub header_1: BeaconBlockHeader,
    pub header_2: BeaconBlockHeader,
}

#[derive(
    Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash, SignedRoot,
)]
pub struct Transfer {
    pub sender: u64,
    pub recipient: u64,
    pub amount: u64,
    pub fee: u64,
    pub slot: Slot,
    pub pubkey: PublicKeyBytes,
    #[signed_root(skip_hashing)]
    pub signature: SignatureBytes,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash)]
pub struct Validator {
    pub pubkey: PublicKeyBytes,
    pub withdrawal_credentials: H256,
    pub effective_balance: u64,
    pub slashed: bool,
    pub activation_eligibility_epoch: Epoch,
    pub activation_epoch: Epoch,
    pub exit_epoch: Epoch,
    pub withdrawable_epoch: Epoch,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            pubkey: PublicKeyBytes::empty(),
            withdrawal_credentials: Default::default(),
            effective_balance: Default::default(),
            slashed: Default::default(),
            activation_eligibility_epoch: Default::default(),
            activation_epoch: Default::default(),
            exit_epoch: Default::default(),
            withdrawable_epoch: Default::default(),
        }
    }
}

#[derive(
    Clone, PartialEq, Eq, Debug, Deserialize, Serialize, SszEncode, SszDecode, TreeHash, SignedRoot,
)]
pub struct VoluntaryExit {
    pub epoch: Epoch,
    pub validator_index: u64,
    #[signed_root(skip_hashing)]
    pub signature: SignatureBytes,
}
