fn main() {
	tonic_build::compile_protos("../proto/hubdj.proto").unwrap();
}
