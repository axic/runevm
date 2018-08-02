extern crate ewasm_api;

#[no_mangle]
pub extern fn main() {
  ewasm_api::revert();
}
