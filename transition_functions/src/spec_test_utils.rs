use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Deserialize};
use ssz::Decode;
use types::{
    beacon_state::BeaconState,
    config::Config,
    primitives::{Slot, H256},
    types::BeaconBlock,
};

#[derive(Deserialize)]
struct BlocksMeta {
    blocks_count: usize,
}

#[derive(Deserialize)]
struct SszMeta {
    root: H256,
}

// TODO(distlt-team): Reword `expect` messages.

pub fn pre<C: Config>(case_directory: impl AsRef<Path>) -> BeaconState<C> {
    ssz(resolve(case_directory).join("pre.ssz")).expect("every test should have a pre-state")
}

pub fn post<C: Config>(case_directory: impl AsRef<Path>) -> Option<BeaconState<C>> {
    ssz(resolve(case_directory).join("post.ssz"))
}

pub fn slots(case_directory: impl AsRef<Path>) -> Slot {
    yaml(resolve(case_directory).join("slots.yaml"))
        .expect("every slot sanity test should have a file specifying the number of slots")
}

pub fn blocks<C: Config>(case_directory: impl AsRef<Path>) -> impl Iterator<Item = BeaconBlock<C>> {
    let BlocksMeta { blocks_count } = yaml(resolve(&case_directory).join("meta.yaml"))
        .expect("every block sanity test should have a file specifying the number of blocks");
    (0..blocks_count).map(move |index| {
        let file_name = format!("blocks_{}.ssz", index);
        ssz(resolve(&case_directory).join(file_name))
            .expect("block sanity tests should have the number of blocks they claim to have")
    })
}

pub fn operation<D: Decode>(
    case_directory: impl AsRef<Path>,
    operation_name: impl AsRef<Path>,
) -> D {
    let operation_path = resolve(case_directory)
        .join(operation_name)
        .with_extension("ssz");
    ssz(operation_path).expect("every operation test should have a file representing the operation")
}

pub fn serialized(case_directory: impl AsRef<Path>) -> Vec<u8> {
    read_optional(resolve(case_directory).join("serialized.ssz"))
        .expect("every SSZ test should have a file with the value encoded in SSZ")
}

pub fn value<D: DeserializeOwned>(case_directory: impl AsRef<Path>) -> D {
    yaml(resolve(case_directory).join("value.yaml"))
        .expect("every SSZ test should have a file with the value encoded in YAML")
}

pub fn root(case_directory: impl AsRef<Path>) -> H256 {
    let SszMeta { root } = yaml(resolve(case_directory).join("roots.yaml"))
        .expect("every SSZ test should have a file specifying the root of the value");
    root
}

fn resolve(case_directory_relative_to_repository_root: impl AsRef<Path>) -> PathBuf {
    // Cargo appears to set the working directory to the crate root when running tests.
    PathBuf::from("..").join(case_directory_relative_to_repository_root)
}

fn ssz<D: Decode>(file_path: impl AsRef<Path>) -> Option<D> {
    let bytes = read_optional(file_path)?;
    let value = D::from_ssz_bytes(bytes.as_slice())
        .expect("the file should contain a value encoded in SSZ");
    Some(value)
}

fn yaml<D: DeserializeOwned>(file_path: impl AsRef<Path>) -> Option<D> {
    let bytes = read_optional(file_path)?;
    let value = serde_yaml::from_slice(bytes.as_slice())
        .expect("the file should contain a value encoded in YAML");
    Some(value)
}

fn read_optional(file_path: impl AsRef<Path>) -> Option<Vec<u8>> {
    match std::fs::read(file_path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => panic!("could not read the file: {:?}", error),
    }
}
