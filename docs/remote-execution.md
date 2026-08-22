# Remote Execution & Distributed Caching

Fish supports high-performance distributed caching and remote execution protocols.

## Remote Execution API (REAPI v2)
Fish integrates Google / Bazel Remote Execution API v2 over gRPC:
```bash
fish build --remote-workers 10.0.0.1:8980,10.0.0.2:8980
```

## P2P Swarm Caching
Enables zero-configuration peer-to-peer artifact sharing over local area networks:
```bash
fish build --swarm
```

## Cloud CAS Backends
Configure S3, GCS, Azure Blob, or Redis storage in `fish.toml`:
```toml
[cache]
enabled = true
remote_url = "s3://my-fish-cache-bucket"
compression = "zstd"
```
