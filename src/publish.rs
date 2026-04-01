use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
#[cfg(feature = "native")]
use std::process::Command;

use crate::{content_cid, ObjectRef, PromotedShardSet, Shard, ShardSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContainerMember {
    pub shard_id: String,
    pub content_digest: String,
    pub member_path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContainerIndex {
    pub artifact_id: String,
    pub artifact_revision: String,
    pub container_id: String,
    pub container_revision: String,
    pub container_encoding: String,
    pub container_object_ref: ObjectRef,
    pub members: Vec<ContainerMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactBundle {
    pub manifest: PromotedShardSet,
    pub container_index: ContainerIndex,
    pub tar_bytes: Vec<u8>,
    pub tar_digest: String,
    #[serde(skip)]
    pub shard_blobs: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublishReceipt {
    pub receipt_id: String,
    pub sink: String,
    pub artifact_id: String,
    pub artifact_revision: String,
    pub shard_refs: Vec<String>,
    pub sink_refs: Vec<ObjectRef>,
    pub container_ref: ObjectRef,
    pub content_digests: Vec<String>,
    pub result: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hosted_acknowledgement: Option<HostedAcknowledgement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublishOutcome {
    pub manifest: PromotedShardSet,
    pub container_index: ContainerIndex,
    pub receipt: PublishReceipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostedAcknowledgement {
    pub sink: String,
    pub acknowledgement_id: String,
    pub locator_uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_url: Option<String>,
    pub content_digest: String,
    pub size_bytes: u64,
    pub verified: bool,
}

impl PublishReceipt {
    pub fn bind_hosted_acknowledgement(
        &mut self,
        acknowledgement: HostedAcknowledgement,
    ) -> Result<(), String> {
        if acknowledgement.sink != self.sink {
            return Err(format!(
                "hosted acknowledgement sink mismatch: {} vs {}",
                acknowledgement.sink, self.sink
            ));
        }
        if acknowledgement.locator_uri != self.container_ref.uri {
            return Err(format!(
                "hosted acknowledgement locator mismatch: {} vs {}",
                acknowledgement.locator_uri, self.container_ref.uri
            ));
        }
        if acknowledgement.content_digest != self.container_ref.content_digest {
            return Err(format!(
                "hosted acknowledgement digest mismatch: {} vs {}",
                acknowledgement.content_digest, self.container_ref.content_digest
            ));
        }
        if acknowledgement.size_bytes != self.container_ref.size_bytes {
            return Err(format!(
                "hosted acknowledgement size mismatch: {} vs {}",
                acknowledgement.size_bytes, self.container_ref.size_bytes
            ));
        }
        self.hosted_acknowledgement = Some(acknowledgement);
        Ok(())
    }
}

impl PublishOutcome {
    pub fn bind_hosted_acknowledgement(
        mut self,
        acknowledgement: HostedAcknowledgement,
    ) -> Result<Self, String> {
        self.receipt.bind_hosted_acknowledgement(acknowledgement)?;
        Ok(self)
    }
}

pub trait SinkAdapter {
    fn sink_name(&self) -> &'static str;
    fn publish(&self, bundle: &ArtifactBundle) -> Result<PublishOutcome, String>;
}

#[derive(Debug, Clone)]
pub struct FileSinkAdapter {
    pub output_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct HfSinkAdapter {
    pub dataset_root: String,
}

#[derive(Debug, Clone)]
pub struct IpfsSinkAdapter;

pub fn build_local_artifact_bundle(
    artifact_id: &str,
    artifact_revision: &str,
    shards: &[Shard],
) -> ArtifactBundle {
    let manifest = PromotedShardSet::from_shards(
        artifact_id,
        artifact_id,
        artifact_revision,
        "erdfa-shard-set",
        "1970-01-01T00:00:00Z",
        shards,
    );

    let mut shard_blobs = BTreeMap::new();
    for shard in shards {
        shard_blobs.insert(shard.id.clone(), shard.to_cbor());
    }

    let shard_set = ShardSet::from_shards(artifact_id, shards);
    let mut tar_bytes = Vec::new();
    shard_set
        .to_tar(shards, &mut tar_bytes)
        .expect("in-memory tar write should succeed");

    let members = shards
        .iter()
        .map(|shard| {
            let blob = shard_blobs
                .get(&shard.id)
                .expect("blob map must contain every shard id");
            ContainerMember {
                shard_id: shard.id.clone(),
                content_digest: digest_with_prefix(blob),
                member_path: format!("{}.cbor", shard.id),
                size_bytes: blob.len() as u64,
            }
        })
        .collect::<Vec<_>>();

    let container_index = ContainerIndex {
        artifact_id: artifact_id.into(),
        artifact_revision: artifact_revision.into(),
        container_id: format!("{artifact_id}-container"),
        container_revision: artifact_revision.into(),
        container_encoding: "tar".into(),
        container_object_ref: ObjectRef {
            sink: "local-bundle".into(),
            uri: format!("bundle://{artifact_id}/{artifact_revision}/container.tar"),
            size_bytes: tar_bytes.len() as u64,
            content_digest: digest_with_prefix(&tar_bytes),
        },
        members,
    };

    ArtifactBundle {
        manifest,
        container_index,
        tar_digest: sha256_hex(&tar_bytes),
        tar_bytes,
        shard_blobs,
    }
}

impl SinkAdapter for FileSinkAdapter {
    fn sink_name(&self) -> &'static str {
        "file"
    }

    fn publish(&self, bundle: &ArtifactBundle) -> Result<PublishOutcome, String> {
        let root = self
            .output_root
            .join(&bundle.container_index.artifact_id)
            .join(&bundle.container_index.artifact_revision);
        let shards_dir = root.join("shards");
        fs::create_dir_all(&shards_dir).map_err(|e| e.to_string())?;

        let container_path = root.join("container.tar");
        fs::write(&container_path, &bundle.tar_bytes).map_err(|e| e.to_string())?;

        let mut refs_by_shard = BTreeMap::new();
        for member in &bundle.container_index.members {
            let blob = bundle
                .shard_blobs
                .get(&member.shard_id)
                .ok_or_else(|| format!("missing blob for {}", member.shard_id))?;
            let computed_digest = digest_with_prefix(blob);
            if computed_digest != member.content_digest {
                return Err(format!(
                    "content digest mismatch for {}: {} vs {}",
                    member.shard_id, computed_digest, member.content_digest
                ));
            }
            let target_path = shards_dir.join(&member.member_path);
            fs::write(&target_path, blob).map_err(|e| e.to_string())?;
            refs_by_shard.insert(
                member.shard_id.clone(),
                ObjectRef {
                    sink: "file".into(),
                    uri: target_path.to_string_lossy().into_owned(),
                    size_bytes: member.size_bytes,
                    content_digest: member.content_digest.clone(),
                },
            );
        }

        let mut manifest = bundle.manifest.clone();
        attach_sink_refs(&mut manifest, "file", &refs_by_shard);

        let mut container_index = bundle.container_index.clone();
        container_index.container_object_ref = ObjectRef {
            sink: "file".into(),
            uri: container_path.to_string_lossy().into_owned(),
            size_bytes: bundle.tar_bytes.len() as u64,
            content_digest: digest_with_prefix(&bundle.tar_bytes),
        };

        let receipt = build_receipt("file", &manifest, &container_index, "published");
        Ok(PublishOutcome {
            manifest,
            container_index,
            receipt,
        })
    }
}

impl SinkAdapter for HfSinkAdapter {
    fn sink_name(&self) -> &'static str {
        "hf"
    }

    fn publish(&self, bundle: &ArtifactBundle) -> Result<PublishOutcome, String> {
        let prefix = format!(
            "{}/{}/{}/shards",
            self.dataset_root.trim_end_matches('/'),
            bundle.container_index.artifact_id,
            bundle.container_index.artifact_revision
        );
        let refs_by_shard = bundle
            .container_index
            .members
            .iter()
            .map(|member| {
                (
                    member.shard_id.clone(),
                    ObjectRef {
                        sink: "hf".into(),
                        uri: format!("{prefix}/{}", member.member_path),
                        size_bytes: member.size_bytes,
                        content_digest: member.content_digest.clone(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();

        let mut manifest = bundle.manifest.clone();
        attach_sink_refs(&mut manifest, "hf", &refs_by_shard);

        let mut container_index = bundle.container_index.clone();
        container_index.container_object_ref = ObjectRef {
            sink: "hf".into(),
            uri: format!(
                "{}/{}/{}/container.tar",
                self.dataset_root.trim_end_matches('/'),
                container_index.artifact_id,
                container_index.artifact_revision
            ),
            size_bytes: bundle.tar_bytes.len() as u64,
            content_digest: digest_with_prefix(&bundle.tar_bytes),
        };

        let receipt = build_receipt("hf", &manifest, &container_index, "projected");
        Ok(PublishOutcome {
            manifest,
            container_index,
            receipt,
        })
    }
}

impl SinkAdapter for IpfsSinkAdapter {
    fn sink_name(&self) -> &'static str {
        "ipfs"
    }

    fn publish(&self, bundle: &ArtifactBundle) -> Result<PublishOutcome, String> {
        let refs_by_shard = bundle
            .container_index
            .members
            .iter()
            .map(|member| {
                let blob = bundle
                    .shard_blobs
                    .get(&member.shard_id)
                    .expect("blob map must contain every member shard id");
                (
                    member.shard_id.clone(),
                    ObjectRef {
                        sink: "ipfs".into(),
                        uri: format!("ipfs://{}", content_cid(blob)),
                        size_bytes: member.size_bytes,
                        content_digest: member.content_digest.clone(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();

        let mut manifest = bundle.manifest.clone();
        attach_sink_refs(&mut manifest, "ipfs", &refs_by_shard);

        let mut container_index = bundle.container_index.clone();
        container_index.container_object_ref = ObjectRef {
            sink: "ipfs".into(),
            uri: format!("ipfs://{}", content_cid(&bundle.tar_bytes)),
            size_bytes: bundle.tar_bytes.len() as u64,
            content_digest: digest_with_prefix(&bundle.tar_bytes),
        };

        let receipt = build_receipt("ipfs", &manifest, &container_index, "projected");
        Ok(PublishOutcome {
            manifest,
            container_index,
            receipt,
        })
    }
}

pub fn apply_sink_adapters(
    bundle: &ArtifactBundle,
    adapters: &[&dyn SinkAdapter],
) -> Result<BTreeMap<String, PublishOutcome>, String> {
    let mut outcomes = BTreeMap::new();
    for adapter in adapters {
        outcomes.insert(adapter.sink_name().to_string(), adapter.publish(bundle)?);
    }
    Ok(outcomes)
}

pub fn write_publish_outcome_artifacts(
    output_root: impl Into<PathBuf>,
    outcome: &PublishOutcome,
) -> Result<PathBuf, String> {
    let root = output_root.into();
    let sink_dir = root
        .join(&outcome.receipt.sink)
        .join(&outcome.receipt.artifact_id)
        .join(&outcome.receipt.artifact_revision);
    fs::create_dir_all(&sink_dir).map_err(|e| e.to_string())?;

    let manifest_path = sink_dir.join("manifest.json");
    let container_index_path = sink_dir.join("container-index.json");
    let receipt_path = sink_dir.join("receipt.json");

    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&outcome.manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        &container_index_path,
        serde_json::to_string_pretty(&outcome.container_index).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        &receipt_path,
        serde_json::to_string_pretty(&outcome.receipt).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(sink_dir)
}

#[cfg(feature = "native")]
pub fn publish_hf_with_ack(
    bundle: &ArtifactBundle,
    adapter: &HfSinkAdapter,
    commit_message: &str,
) -> Result<PublishOutcome, String> {
    let mut outcome = adapter.publish(bundle)?;
    let local_path = write_temp_tar(bundle, "hf-upload")?;
    let (repo_id, object_path) = parse_hf_container_uri(&outcome.receipt.container_ref.uri)?;
    let output = Command::new("hf")
        .args([
            "upload",
            &repo_id,
            local_path.to_string_lossy().as_ref(),
            &object_path,
            "--repo-type",
            "dataset",
            "--commit-message",
            commit_message,
        ])
        .output()
        .map_err(|e| format!("hf upload failed to start: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "hf upload failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let revision = parse_hf_commit_revision(&combined)?;
    let verify_url = format!(
        "https://huggingface.co/datasets/{repo_id}/resolve/{revision}/{object_path}"
    );
    let response = ureq::get(&verify_url)
        .call()
        .map_err(|e| format!("hf verify fetch failed: {e}"))?;
    let bytes = response
        .into_reader()
        .bytes()
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| format!("hf verify read failed: {e}"))?;
    let ack = HostedAcknowledgement {
        sink: "hf".into(),
        acknowledgement_id: revision.clone(),
        locator_uri: outcome.receipt.container_ref.uri.clone(),
        verification_url: Some(format!("https://huggingface.co/datasets/{repo_id}/commit/{revision}")),
        content_digest: digest_with_prefix(&bytes),
        size_bytes: bytes.len() as u64,
        verified: digest_with_prefix(&bytes) == outcome.receipt.container_ref.content_digest,
    };
    let _ = fs::remove_file(&local_path);
    outcome.bind_hosted_acknowledgement(ack)
}

#[cfg(feature = "native")]
pub fn publish_ipfs_with_ack(
    bundle: &ArtifactBundle,
    adapter: &IpfsSinkAdapter,
    api_base_url: &str,
    gateway_base_url: &str,
) -> Result<PublishOutcome, String> {
    let mut outcome = adapter.publish(bundle)?;
    let local_path = write_temp_tar(bundle, "ipfs-upload")?;
    let add_url = format!("{}/api/v0/add?pin=true", api_base_url.trim_end_matches('/'));
    let output = Command::new("curl")
        .args([
            "-sS",
            "-X",
            "POST",
            &add_url,
            "-F",
            &format!("file=@{}", local_path.to_string_lossy()),
        ])
        .output()
        .map_err(|e| format!("ipfs add failed to start: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "ipfs add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let body = String::from_utf8_lossy(&output.stdout).to_string();
    let cid = parse_ipfs_cid(&body)?;
    let actual_uri = format!("ipfs://{cid}");
    let verify_url = format!("{}/ipfs/{}", gateway_base_url.trim_end_matches('/'), cid);
    let verify_response = ureq::get(&verify_url)
        .call()
        .map_err(|e| format!("ipfs verify fetch failed: {e}"))?;
    let bytes = verify_response
        .into_reader()
        .bytes()
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| format!("ipfs verify read failed: {e}"))?;
    let actual_ref = ObjectRef {
        sink: "ipfs".into(),
        uri: actual_uri.clone(),
        size_bytes: bytes.len() as u64,
        content_digest: digest_with_prefix(&bytes),
    };
    outcome.container_index.container_object_ref = actual_ref.clone();
    outcome.receipt.container_ref = actual_ref.clone();
    outcome.receipt.result = format!("published:{}", actual_uri);
    let ack = HostedAcknowledgement {
        sink: "ipfs".into(),
        acknowledgement_id: cid.clone(),
        locator_uri: actual_uri,
        verification_url: Some(verify_url),
        content_digest: actual_ref.content_digest.clone(),
        size_bytes: actual_ref.size_bytes,
        verified: actual_ref.content_digest == digest_with_prefix(&bundle.tar_bytes),
    };
    let _ = fs::remove_file(&local_path);
    outcome.bind_hosted_acknowledgement(ack)
}

fn attach_sink_refs(
    manifest: &mut PromotedShardSet,
    sink: &str,
    refs_by_shard: &BTreeMap<String, ObjectRef>,
) {
    for shard in &mut manifest.shards {
        if let Some(reference) = refs_by_shard.get(&shard.id) {
            shard.object_refs.retain(|r| r.sink != sink);
            shard.object_refs.push(reference.clone());
        }
    }
}

fn build_receipt(
    sink: &str,
    manifest: &PromotedShardSet,
    container_index: &ContainerIndex,
    result: &str,
) -> PublishReceipt {
    let sink_refs = manifest
        .shards
        .iter()
        .flat_map(|s| s.object_refs.iter().filter(|r| r.sink == sink).cloned())
        .collect::<Vec<_>>();
    let content_digests = sink_refs
        .iter()
        .map(|r| r.content_digest.clone())
        .collect::<Vec<_>>();

    PublishReceipt {
        receipt_id: format!(
            "receipt:{}:{}:{}",
            sink, manifest.artifact_id, manifest.artifact_revision
        ),
        sink: sink.into(),
        artifact_id: manifest.artifact_id.clone(),
        artifact_revision: manifest.artifact_revision.clone(),
        shard_refs: manifest.shards.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        sink_refs,
        container_ref: container_index.container_object_ref.clone(),
        content_digests,
        result: format!("{result}:{}", container_index.container_object_ref.uri),
        hosted_acknowledgement: None,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn digest_with_prefix(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

#[cfg(feature = "native")]
fn write_temp_tar(bundle: &ArtifactBundle, prefix: &str) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!(
        "erdfa-publish-{}-{}-{}-{}.tar",
        prefix,
        bundle.container_index.artifact_id,
        bundle.container_index.artifact_revision,
        std::process::id()
    ));
    fs::write(&path, &bundle.tar_bytes).map_err(|e| format!("temp tar write failed: {e}"))?;
    Ok(path)
}

#[cfg(feature = "native")]
fn parse_hf_container_uri(uri: &str) -> Result<(String, String), String> {
    let payload = uri
        .strip_prefix("hf://datasets/")
        .ok_or_else(|| format!("unsupported HF dataset uri: {uri}"))?;
    let mut parts = payload.split('/').collect::<Vec<_>>();
    if parts.len() < 3 {
        return Err(format!("HF dataset uri missing path components: {uri}"));
    }
    let repo_id = format!("{}/{}", parts[0], parts[1]);
    let object_path = parts.drain(2..).collect::<Vec<_>>().join("/");
    Ok((repo_id, object_path))
}

#[cfg(feature = "native")]
fn parse_hf_commit_revision(output: &str) -> Result<String, String> {
    output
        .split("/commit/")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() == 40)
        .ok_or_else(|| format!("unable to parse HF commit revision from output: {output}"))
}

#[cfg(feature = "native")]
fn parse_ipfs_cid(output: &str) -> Result<String, String> {
    if let Some(hash_field) = output.split("\"Hash\":\"").nth(1) {
        return Ok(hash_field
            .split('"')
            .next()
            .unwrap_or_default()
            .to_string());
    }
    output
        .split_whitespace()
        .find(|part| part.starts_with("Qm") || part.starts_with("bafy"))
        .map(|s| s.to_string())
        .ok_or_else(|| format!("unable to parse IPFS CID from output: {output}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Component, Shard};

    fn demo_shards() -> Vec<Shard> {
        vec![
            Shard::new(
                "heading-1",
                Component::Heading {
                    level: 1,
                    text: "Demo".into(),
                },
            ),
            Shard::new(
                "left-bucket-001",
                Component::Paragraph {
                    text: "Left bucket".into(),
                },
            ),
            Shard::new(
                "nodeOfName-en-017",
                Component::Paragraph {
                    text: "tenant".into(),
                },
            ),
        ]
    }

    #[test]
    fn local_bundle_is_stable_and_membership_matches_manifest() {
        let shards = demo_shards();
        let bundle_a = build_local_artifact_bundle("artifact-demo", "rev-a", &shards);
        let bundle_b = build_local_artifact_bundle("artifact-demo", "rev-a", &shards);

        assert_eq!(bundle_a.manifest.shards.len(), shards.len());
        assert_eq!(bundle_a.tar_digest, bundle_b.tar_digest);

        for member in &bundle_a.container_index.members {
            let shard = bundle_a
                .manifest
                .shards
                .iter()
                .find(|s| s.id == member.shard_id)
                .expect("container member shard must exist");
            assert_eq!(member.size_bytes, shard.size_bytes);
            assert_eq!(member.member_path, format!("{}.cbor", shard.id));
        }
    }

    #[test]
    fn file_sink_publishes_and_receipt_matches_written_content() {
        let root = std::env::temp_dir().join(format!("erdfa-publish-test-{}", std::process::id()));
        let shards = demo_shards();
        let bundle = build_local_artifact_bundle("artifact-demo", "rev-a", &shards);
        let adapter = FileSinkAdapter {
            output_root: root.clone(),
        };

        let outcome = adapter.publish(&bundle).expect("file publish should work");

        assert_eq!(outcome.receipt.sink, "file");
        assert_eq!(outcome.receipt.shard_refs.len(), shards.len());
        assert_eq!(outcome.receipt.sink_refs.len(), shards.len());

        for shard in &outcome.manifest.shards {
            let file_ref = shard
                .object_refs
                .iter()
                .find(|r| r.sink == "file")
                .expect("file object ref must be present");
            let written = fs::read(&file_ref.uri).expect("written shard should exist");
            assert_eq!(file_ref.content_digest, digest_with_prefix(&written));
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn hf_and_ipfs_adapters_attach_refs_without_mutating_shard_identity() {
        let shards = demo_shards();
        let bundle = build_local_artifact_bundle("artifact-demo", "rev-a", &shards);

        let hf = HfSinkAdapter {
            dataset_root: "hf://datasets/example/zelph".into(),
        };
        let ipfs = IpfsSinkAdapter;

        let hf_outcome = hf.publish(&bundle).expect("hf projection should work");
        let ipfs_outcome = ipfs.publish(&bundle).expect("ipfs projection should work");

        let baseline = bundle
            .manifest
            .shards
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>();
        let hf_ids = hf_outcome
            .manifest
            .shards
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>();
        let ipfs_ids = ipfs_outcome
            .manifest
            .shards
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>();

        assert_eq!(baseline, hf_ids);
        assert_eq!(baseline, ipfs_ids);
        assert!(hf_outcome
            .manifest
            .shards
            .iter()
            .all(|s| s.object_refs.iter().any(|r| r.sink == "hf")));
        assert!(ipfs_outcome
            .manifest
            .shards
            .iter()
            .all(|s| s.object_refs.iter().any(|r| r.sink == "ipfs")));
    }

    #[test]
    fn workflow_applies_all_adapters_and_keeps_container_membership_in_sync() {
        let shards = demo_shards();
        let bundle = build_local_artifact_bundle("artifact-demo", "rev-a", &shards);
        let file = FileSinkAdapter {
            output_root: std::env::temp_dir().join(format!("erdfa-publish-all-{}", std::process::id())),
        };
        let hf = HfSinkAdapter {
            dataset_root: "hf://datasets/example/zelph".into(),
        };
        let ipfs = IpfsSinkAdapter;

        let outcomes = apply_sink_adapters(&bundle, &[&file, &hf, &ipfs]).expect("workflow must succeed");
        assert_eq!(outcomes.len(), 3);

        let file_outcome = outcomes.get("file").expect("file outcome missing");
        assert_eq!(file_outcome.container_index.members.len(), file_outcome.manifest.shards.len());

        let _ = fs::remove_dir_all(&file.output_root);
    }

    #[test]
    fn hosted_acknowledgement_binds_to_hf_receipt_when_container_matches() {
        let shards = demo_shards();
        let bundle = build_local_artifact_bundle("artifact-demo", "rev-a", &shards);
        let hf = HfSinkAdapter {
            dataset_root: "hf://datasets/example/zelph".into(),
        };

        let outcome = hf.publish(&bundle).expect("hf projection should work");
        let ack = HostedAcknowledgement {
            sink: "hf".into(),
            acknowledgement_id: "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
            locator_uri: outcome.receipt.container_ref.uri.clone(),
            verification_url: Some("https://huggingface.co/datasets/example/zelph/commit/deadbeef".into()),
            content_digest: outcome.receipt.container_ref.content_digest.clone(),
            size_bytes: outcome.receipt.container_ref.size_bytes,
            verified: true,
        };

        let enriched = outcome
            .bind_hosted_acknowledgement(ack.clone())
            .expect("hosted acknowledgement should bind");
        assert_eq!(enriched.receipt.hosted_acknowledgement, Some(ack));
    }

    #[test]
    fn hosted_acknowledgement_rejects_mismatched_digest() {
        let shards = demo_shards();
        let bundle = build_local_artifact_bundle("artifact-demo", "rev-a", &shards);
        let ipfs = IpfsSinkAdapter;

        let outcome = ipfs.publish(&bundle).expect("ipfs projection should work");
        let ack = HostedAcknowledgement {
            sink: "ipfs".into(),
            acknowledgement_id: "QmMismatch".into(),
            locator_uri: outcome.receipt.container_ref.uri.clone(),
            verification_url: Some("http://127.0.0.1:8080/ipfs/QmMismatch".into()),
            content_digest: "sha256:deadbeef".into(),
            size_bytes: outcome.receipt.container_ref.size_bytes,
            verified: true,
        };

        let err = outcome
            .bind_hosted_acknowledgement(ack)
            .expect_err("mismatched digest must fail");
        assert!(err.contains("digest mismatch"));
    }

    #[test]
    fn publish_outcome_artifacts_are_written() {
        let shards = demo_shards();
        let bundle = build_local_artifact_bundle("artifact-demo", "rev-a", &shards);
        let hf = HfSinkAdapter {
            dataset_root: "hf://datasets/example/zelph".into(),
        };
        let outcome = hf.publish(&bundle).expect("hf projection should work");
        let root = std::env::temp_dir().join(format!("erdfa-publish-artifacts-{}", std::process::id()));
        let sink_dir = write_publish_outcome_artifacts(&root, &outcome).expect("artifact write should work");
        assert!(sink_dir.join("manifest.json").exists());
        assert!(sink_dir.join("container-index.json").exists());
        assert!(sink_dir.join("receipt.json").exists());
        let _ = fs::remove_dir_all(root);
    }
}
