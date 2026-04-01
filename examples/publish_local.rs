use erdfa_publish::{
    apply_sink_adapters, build_local_artifact_bundle, Component, FileSinkAdapter, HfSinkAdapter,
    HostedAcknowledgement, IpfsSinkAdapter, Shard,
};

fn main() {
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

    let file = FileSinkAdapter {
        output_root: std::env::temp_dir().join("erdfa-publish-local"),
    };
    let hf = HfSinkAdapter {
        dataset_root: "hf://datasets/example/zelph".into(),
    };
    let ipfs = IpfsSinkAdapter;

    let outcomes = apply_sink_adapters(&bundle, &[&file, &hf, &ipfs]).expect("publish workflow");
    for (sink, outcome) in outcomes {
        let outcome = if sink == "hf" || sink == "ipfs" {
            let acknowledgement = HostedAcknowledgement {
                sink: sink.clone(),
                acknowledgement_id: format!("demo-ack-{}", sink),
                locator_uri: outcome.receipt.container_ref.uri.clone(),
                verification_url: Some(format!("demo://{}/verify", sink)),
                content_digest: outcome.receipt.container_ref.content_digest.clone(),
                size_bytes: outcome.receipt.container_ref.size_bytes,
                verified: true,
            };
            outcome
                .bind_hosted_acknowledgement(acknowledgement)
                .expect("demo acknowledgement should bind")
        } else {
            outcome
        };
        println!(
            "{} -> receipt={} refs={} hosted_ack={}",
            sink,
            outcome.receipt.receipt_id,
            outcome.receipt.sink_refs.len(),
            outcome.receipt.hosted_acknowledgement.is_some()
        );
    }
}
