use libcwiid_sys::bdaddr_t;


#[derive(Debug)]
pub struct Address {
    address: bdaddr_t,
}

impl Default for Address {
    fn default() -> Self {
        Self { address: bdaddr_t { b: [0,0,0,0,0,0] } }
    }
}

impl AsMut<bdaddr_t> for Address {
    fn as_mut(&mut self) -> &mut bdaddr_t {
        &mut self.address
    }
}