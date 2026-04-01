use erdfa_publish::{
    build_local_artifact_bundle, publish_hf_with_ack, publish_ipfs_with_ack,
    write_publish_outcome_artifacts, Component, HfSinkAdapter, IpfsSinkAdapter, Shard,
};

fn main() {
    let hf_root = std::env::var("ERDFA_HF_DATASET_ROOT")
        .expect("set ERDFA_HF_DATASET_ROOT, e.g. hf://datasets/chbwa/itir-zos-ack-probe");
    let ipfs_api =
        std::env::var("ERDFA_IPFS_API").unwrap_or_else(|_| "http://127.0.0.1:5001".into());
    let ipfs_gateway =
        std::env::var("ERDFA_IPFS_GATEWAY").unwrap_or_else(|_| "http://127.0.0.1:8080".into());

    let shards = vec![
        Shard::new(
            "left-bucket-001",
            Component::Paragraph {
                text: "left node bucket".into(),
            },
        ),
        Shard::new(
            "nodeOfName-en-017",
            Component::Paragraph {
                text: "tenant".into(),
            },
        ),
    ];

    let bundle = build_local_artifact_bundle("artifact-demo", "rev-a", &shards);

    let hf = HfSinkAdapter {
        dataset_root: hf_root,
    };
    let ipfs = IpfsSinkAdapter;

    let hf_outcome =
        publish_hf_with_ack(&bundle, &hf, "Publish hosted ERDFA demo bundle").expect("hf hosted publish");
    let ipfs_outcome =
        publish_ipfs_with_ack(&bundle, &ipfs, &ipfs_api, &ipfs_gateway).expect("ipfs hosted publish");
    let output_root =
        std::env::var("ERDFA_PUBLISH_OUTPUT_ROOT").unwrap_or_else(|_| "/tmp/erdfa-publish-hosted".into());
    let hf_dir =
        write_publish_outcome_artifacts(&output_root, &hf_outcome).expect("hf artifact write");
    let ipfs_dir =
        write_publish_outcome_artifacts(&output_root, &ipfs_outcome).expect("ipfs artifact write");

    println!(
        "hf -> ack={} verified={} dir={}",
        hf_outcome
            .receipt
            .hosted_acknowledgement
            .as_ref()
            .map(|ack| ack.acknowledgement_id.as_str())
            .unwrap_or("missing"),
        hf_outcome
            .receipt
            .hosted_acknowledgement
            .as_ref()
            .map(|ack| ack.verified)
            .unwrap_or(false),
        hf_dir.display()
    );
    println!(
        "ipfs -> ack={} verified={} dir={}",
        ipfs_outcome
            .receipt
            .hosted_acknowledgement
            .as_ref()
            .map(|ack| ack.acknowledgement_id.as_str())
            .unwrap_or("missing"),
        ipfs_outcome
            .receipt
            .hosted_acknowledgement
            .as_ref()
            .map(|ack| ack.verified)
            .unwrap_or(false),
        ipfs_dir.display()
    );
}
