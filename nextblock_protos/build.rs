//use tonic_build::configure;

fn main() {
    let _ = tonic_build::compile_protos("proto/api.proto");
}
