pub mod generated {
    include!("generated/google.protobuf.rs");
    include!("generated/swarm_sync.rs");
}

// You can optionally move test_grpc_connection here if you want to keep CLI and tests DRY
pub mod grpc;
