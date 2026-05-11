use communication_core::NodeDescriptor;

#[derive(Clone)]
pub struct LocalNodeIdentity {
    pub descriptor: NodeDescriptor,
    pub private_key_pkcs8: Vec<u8>,
}
