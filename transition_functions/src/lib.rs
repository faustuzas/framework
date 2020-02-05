pub mod attestations;
pub mod blocks;
pub mod epochs;
pub mod process_slot;
pub mod rewards_and_penalties;

#[cfg(test)]
mod spec_test_utils;

// CONSIDER(distlt-team): Rename other spec test modules or functions.

// CONSIDER(distlt-team): Rename the test module.
// COMMENT(distlt-team): Why the SSZ tests are here.
#[cfg(test)]
mod ssz_spec_tests {
    use core::fmt::Debug;

    use helper_functions::crypto;
    use serde::de::DeserializeOwned;
    use ssz::{Decode, Encode};
    use test_generator::test_resources;
    use tree_hash::TreeHash;
    use types::{
        beacon_state::BeaconState,
        config::MinimalConfig,
        types::{
            Attestation, AttestationData, AttesterSlashing, BeaconBlock, BeaconBlockBody,
            BeaconBlockHeader, Checkpoint, Deposit, DepositData, Eth1Data, Fork, HistoricalBatch,
            IndexedAttestation, PendingAttestation, ProposerSlashing, Validator, VoluntaryExit,
        },
    };

    use crate::spec_test_utils;

    // COMMENT(distlt-team): We don't generate tests for `AggregateAndProof` because we don't have a
    //                       validator client implementation.

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Attestation/*/*")]
    fn minimal_attestation(case_directory: &str) {
        run_case::<Attestation<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/AttestationData/*/*")]
    fn minimal_attestation_data(case_directory: &str) {
        run_case::<AttestationData>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/AttesterSlashing/*/*")]
    fn minimal_attester_slashing(case_directory: &str) {
        run_case::<AttesterSlashing<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconBlock/*/*")]
    fn minimal_beacon_block(case_directory: &str) {
        run_case::<BeaconBlock<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconBlockBody/*/*")]
    fn minimal_beacon_block_body(case_directory: &str) {
        run_case::<BeaconBlockBody<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconBlockHeader/*/*")]
    fn minimal_beacon_block_header(case_directory: &str) {
        run_case::<BeaconBlockHeader>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/BeaconState/*/*")]
    fn minimal_beacon_state(case_directory: &str) {
        run_case::<BeaconState<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Checkpoint/*/*")]
    fn minimal_checkpoint(case_directory: &str) {
        run_case::<Checkpoint>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Deposit/*/*")]
    fn minimal_deposit(case_directory: &str) {
        run_case::<Deposit>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/DepositData/*/*")]
    fn minimal_deposit_data(case_directory: &str) {
        run_case::<DepositData>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/*/*")]
    fn minimal_eth1_data(case_directory: &str) {
        run_case::<Eth1Data>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Fork/*/*")]
    fn minimal_fork(case_directory: &str) {
        run_case::<Fork>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/HistoricalBatch/*/*")]
    fn minimal_historical_batch(case_directory: &str) {
        run_case::<HistoricalBatch<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/IndexedAttestation/*/*")]
    fn minimal_indexed_attestation(case_directory: &str) {
        run_case::<IndexedAttestation<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/PendingAttestation/*/*")]
    fn minimal_pending_attestation(case_directory: &str) {
        run_case::<PendingAttestation<MinimalConfig>>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/ProposerSlashing/*/*")]
    fn minimal_proposer_slashing(case_directory: &str) {
        run_case::<ProposerSlashing>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/Validator/*/*")]
    fn minimal_validator(case_directory: &str) {
        run_case::<Validator>(case_directory);
    }

    #[test_resources("eth2.0-spec-tests/tests/minimal/phase0/ssz_static/VoluntaryExit/*/*")]
    fn minimal_voluntary_exit(case_directory: &str) {
        run_case::<VoluntaryExit>(case_directory);
    }

    fn run_case<D: PartialEq + Debug + DeserializeOwned + Decode + Encode + TreeHash>(
        case_directory: &str,
    ) {
        let ssz_bytes = spec_test_utils::serialized(case_directory);
        let yaml_value = spec_test_utils::value(case_directory);
        let root = spec_test_utils::root(case_directory);
        let ssz_value = D::from_ssz_bytes(ssz_bytes.as_slice())
            .expect("the SSZ file should contain valid data");
        assert_eq!(ssz_value, yaml_value);
        assert_eq!(ssz_bytes, yaml_value.as_ssz_bytes());
        assert_eq!(crypto::hash_tree_root(&yaml_value), root);
    }
}
