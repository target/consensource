use crypto::digest::Digest;
use crypto::sha2::Sha256;

pub const FAMILY_NAMESPACE: &str = "consensource";
pub const FAMILY_VERSION: &str = "0.1";
const AGENT: &str = "00";
const CERTIFICATE: &str = "01";
const ORGANIZATION: &str = "02";
const STANDARD: &str = "03";
const REQUEST: &str = "04";

const PREFIX_SIZE: usize = 6;
const RESERVED_SPACE: &str = "00";

fn hash(object: &str, num: usize) -> String {
    let mut sha = Sha256::new();
    sha.input_str(object);
    sha.result_str()[..num].to_string()
}

/// Calculates and returns the first 6 digit hex of the family namespace Sha-2
pub fn get_family_namespace_prefix() -> String {
    hash(&FAMILY_NAMESPACE, PREFIX_SIZE)
}

/// Returns the address for an agent based on the provided public key
pub fn make_agent_address(agent_public_key: &str) -> String {
    get_family_namespace_prefix() + RESERVED_SPACE + AGENT + &hash(agent_public_key, 60)
}

/// Returns the address for an organization based on the provided organization id
pub fn make_organization_address(organization_id: &str) -> String {
    get_family_namespace_prefix() + RESERVED_SPACE + ORGANIZATION + &hash(organization_id, 60)
}

/// Returns the address for a certificate based on the provided certificate id
pub fn make_certificate_address(certificate_id: &str) -> String {
    get_family_namespace_prefix() + RESERVED_SPACE + CERTIFICATE + &hash(certificate_id, 60)
}

/// Returns the address for a request based on the provided request id
pub fn make_request_address(request_id: &str) -> String {
    get_family_namespace_prefix() + RESERVED_SPACE + REQUEST + &hash(request_id, 60)
}

/// Returns the address for a request based on the provided request id
pub fn make_standard_address(standard_id: &str) -> String {
    get_family_namespace_prefix() + RESERVED_SPACE + STANDARD + &hash(standard_id, 60)
}

#[derive(Debug)]
pub enum AddressSpace {
    Organization,
    Agent,
    Certificate,
    Request,
    Standard,
    AnotherFamily,
}

/// that takes in an address from state, and
/// returns the kind of state object that address
/// maps to
pub fn get_address_type(address: &str) -> AddressSpace {
    let infix = &address[8..10];

    if infix == AGENT {
        AddressSpace::Agent
    } else if infix == CERTIFICATE {
        AddressSpace::Certificate
    } else if infix == ORGANIZATION {
        AddressSpace::Organization
    } else if infix == REQUEST {
        AddressSpace::Request
    } else if infix == STANDARD {
        AddressSpace::Standard
    } else {
        AddressSpace::AnotherFamily
    }
}
