fn main() {
    tonic_build::compile_protos("proto/api.proto").expect("failed to compile NextBlock protobuf schema");
}
