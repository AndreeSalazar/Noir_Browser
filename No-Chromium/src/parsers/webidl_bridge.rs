// AUTO-GENERATED WebIDL BINDINGS
// This connects JS calls to Rust native pointers
pub struct WebIDLBridge;
impl WebIDLBridge {
    pub fn query_selector(_selector: &str) { println!("[WebIDL] querySelector invoked"); }
    pub fn append_child(_node: u64) { println!("[WebIDL] appendChild invoked"); }
}