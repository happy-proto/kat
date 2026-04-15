set shell := ["bash", "-cu"]

etcd-parquet-sizes env="prod":
    @cd mdsre && \
    uv run mdsre mongo query --env {{env}} 'const rows=db.getSiblingDB("dc").etcd_lease_parquet_artifact.find({}, {_id:0, concrete_cluster:1, parquet_bytes:1, export_started_at:1, expire_at:1}).sort({parquet_bytes:-1}).toArray(); print(["parquet_bytes","concrete_cluster","export_started_at","expire_at"].join("\t")); rows.forEach(r => print([Number(r.parquet_bytes), r.concrete_cluster, r.export_started_at.toISOString(), r.expire_at.toISOString()].join("\t")));' | column -t -s $'\t'

lint:
    cargo fmt --check
    cargo clippy --all-targets --all-features
