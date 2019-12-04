/*
 * CertState
 */

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
        use sabre_sdk::TransactionContext;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
    }
}

use std::collections::HashMap;

use common::addressing;

use common::proto;
use protobuf;

pub struct CertState<'a> {
    context: &'a mut TransactionContext,
}

impl<'a> CertState<'a> {
    // Create new instance of CertState
    pub fn new(context: &'a mut TransactionContext) -> CertState {
        CertState { context }
    }

    /// Fetches and deserializes an Agent's data from state
    /// ```
    /// # Erros
    /// Return an error if it fails to deserialize the Agent's data
    /// ```
    pub fn get_agent(
        &mut self,
        agent_public_key: &str,
    ) -> Result<Option<proto::agent::Agent>, ApplyError> {
        let address = addressing::make_agent_address(agent_public_key);
        let state_data = self.context.get_state(vec![address])?;
        match state_data {
            Some(data) => {
                let agents: proto::agent::AgentContainer =
                    protobuf::parse_from_bytes(data.as_slice()).map_err(|_err| {
                        ApplyError::InvalidTransaction(String::from(
                            "Cannot deserialize agent container",
                        ))
                    })?;

                for agent in agents.get_entries() {
                    if agent.public_key == agent_public_key {
                        return Ok(Some(agent.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Fetches and deserializes an Organization's data from state
    /// ```
    /// # Erros
    /// Return an error if it fails to deserialize the Organization's data
    /// ```
    pub fn get_organization(
        &mut self,
        organization_id: &str,
    ) -> Result<Option<proto::organization::Organization>, ApplyError> {
        let address = addressing::make_organization_address(organization_id);
        let state_data = self.context.get_state(vec![address])?;
        match state_data {
            Some(data) => {
                let organizations: proto::organization::OrganizationContainer =
                    protobuf::parse_from_bytes(data.as_slice()).map_err(|_err| {
                        ApplyError::InvalidTransaction(String::from(
                            "Cannot deserialize organization container",
                        ))
                    })?;

                for organization in organizations.get_entries() {
                    if organization.id == organization_id {
                        return Ok(Some(organization.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Fetches and deserializes a Certificate's data from state
    /// ```
    /// # Erros
    /// Return an error if it fails to deserialize the Certificate's data
    /// ```
    pub fn get_certificate(
        &mut self,
        certificate_id: &str,
    ) -> Result<Option<proto::certificate::Certificate>, ApplyError> {
        let address = addressing::make_certificate_address(certificate_id);
        let state_data = self.context.get_state(vec![address])?;
        match state_data {
            Some(data) => {
                let certificates: proto::certificate::CertificateContainer =
                    protobuf::parse_from_bytes(data.as_slice()).map_err(|_err| {
                        ApplyError::InvalidTransaction(String::from(
                            "Cannot deserialize certificate container",
                        ))
                    })?;

                for certificate in certificates.get_entries() {
                    if certificate.id == certificate_id {
                        return Ok(Some(certificate.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Fetches and deserializes a Request data from state
    /// ```
    /// # Erros
    /// Return an error if it fails to deserialize the Request's data
    /// ```
    pub fn get_request(
        &mut self,
        request_id: &str,
    ) -> Result<Option<proto::request::Request>, ApplyError> {
        let address = addressing::make_request_address(request_id);
        let state_data = self.context.get_state(vec![address])?;
        match state_data {
            Some(data) => {
                let open_requests: proto::request::RequestContainer =
                    protobuf::parse_from_bytes(data.as_slice()).map_err(|_err| {
                        ApplyError::InvalidTransaction(String::from(
                            "Cannot deserialize Request container",
                        ))
                    })?;

                for open_request in open_requests.get_entries() {
                    if open_request.id == request_id {
                        return Ok(Some(open_request.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Fetches and deserializes a Standard data from state
    /// ```
    /// # Erros
    /// Return an error if it fails to deserialize the Standard's data
    /// ```
    pub fn get_standard(
        &mut self,
        standard_id: &str,
    ) -> Result<Option<proto::standard::Standard>, ApplyError> {
        let address = addressing::make_standard_address(standard_id);
        let state_data = self.context.get_state(vec![address])?;
        match state_data {
            Some(data) => {
                let standards: proto::standard::StandardContainer =
                    protobuf::parse_from_bytes(data.as_slice()).map_err(|_err| {
                        ApplyError::InvalidTransaction(String::from(
                            "Cannot deserialize Standard container",
                        ))
                    })?;

                for standard in standards.get_entries() {
                    if standard.id == standard_id {
                        return Ok(Some(standard.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// As the addressing scheme does not guarantee uniquesness, this adds an Agent into a Agent Container
    /// which works like a hashbucket, serializes the container and puts it into state,
    /// ```
    /// # Errors
    /// Returns an error if it fails to serialize the container or fails to set it to state
    /// ```
    pub fn set_agent(
        &mut self,
        agent_public_key: &str,
        agent: proto::agent::Agent,
    ) -> Result<(), ApplyError> {
        let address = addressing::make_agent_address(agent_public_key);
        let state_data = self.context.get_state(vec![address.clone()])?;
        let mut agents: proto::agent::AgentContainer = match state_data {
            Some(data) => protobuf::parse_from_bytes(data.as_slice()).map_err(|_err| {
                ApplyError::InvalidTransaction(String::from("Cannot deserialize agent container"))
            })?,
            // If there nothing at that memory address in state, make a new container, and create a new agent
            None => proto::agent::AgentContainer::new(),
        };

        // Use an iterator to find the index of an agent pub key that matches the agent attempting to be created
        if let Some((i, _)) = agents
            .entries
            .iter()
            .enumerate()
            .find(|(_i, agent)| agent.public_key == agent_public_key)
        {
            // If that agent already exists, set agents_slice to that agent
            let agent_slice = agents.entries.as_mut_slice();
            agent_slice[i] = agent;
        } else {
            // Push new and unique agent to the AgentContainer
            agents.entries.push(agent);
            agents.entries.sort_by_key(|a| a.clone().public_key);
        }

        let serialized = protobuf::Message::write_to_bytes(&agents).map_err(|_err| {
            ApplyError::InvalidTransaction(String::from("Cannot serialize agent container"))
        })?;

        // Insert serialized AgentContainer to an address in the merkle tree
        let mut sets = HashMap::new();
        sets.insert(address, serialized);
        self.context.set_state(sets)?;
        Ok(())
    }

    /// As the addressing scheme does not guarantee uniquesness, this adds an Organization into a Organization Container
    /// which works like a hashbucket, serializes the container and puts it into state,
    /// ```
    /// # Errors
    /// Returns an error if it fails to serialize the container or fails to set it to state
    /// ```
    pub fn set_organization(
        &mut self,
        organization_id: &str,
        organization: proto::organization::Organization,
    ) -> Result<(), ApplyError> {
        let address = addressing::make_organization_address(organization_id);
        let state_data = self.context.get_state(vec![address.clone()])?;
        let mut organizations: proto::organization::OrganizationContainer = match state_data {
            Some(data) => protobuf::parse_from_bytes(data.as_slice()).map_err(|_err| {
                ApplyError::InvalidTransaction(String::from(
                    "Cannot deserialize organization container",
                ))
            })?,
            // If there is nothing at that memory address in state, make a new container, and create a new organization
            None => proto::organization::OrganizationContainer::new(),
        };

        // Use an iterator to find the index of an organization's ID that matches the organization attempting to be created
        if let Some((i, _)) = organizations
            .entries
            .iter()
            .enumerate()
            .find(|(_i, organization)| organization.id == organization_id)
        {
            // If that organization already exists, set organization_slice to that organization
            let organization_slice = organizations.entries.as_mut_slice();
            organization_slice[i] = organization;
        } else {
            // Push new and unique organization to the OrganizationContainer
            organizations.entries.push(organization);
            organizations.entries.sort_by_key(|o| o.clone().id);
        }

        let serialized = protobuf::Message::write_to_bytes(&organizations).map_err(|_err| {
            ApplyError::InvalidTransaction(String::from("Cannot serialize organization container"))
        })?;

        // Insert serialized OrganizationContainer to an address in the merkle tree
        let mut sets = HashMap::new();
        sets.insert(address, serialized);
        self.context.set_state(sets)?;
        Ok(())
    }

    /// As the addressing scheme does not guarantee uniquesness, this adds a Certificate into a Certificate Container
    /// which works like a hashbucket, serializes the container and puts it into state,
    /// ```
    /// # Errors
    /// Returns an error if it fails to serialize the container or fails to set it to state
    /// ```
    pub fn set_certificate(
        &mut self,
        certificate_id: &str,
        certificate: proto::certificate::Certificate,
    ) -> Result<(), ApplyError> {
        let address = addressing::make_certificate_address(certificate_id);
        let state_data = self.context.get_state(vec![address.clone()])?;
        let mut certificates: proto::certificate::CertificateContainer = match state_data {
            Some(data) => protobuf::parse_from_bytes(data.as_slice()).map_err(|_err| {
                ApplyError::InvalidTransaction(String::from(
                    "Cannot deserialize certificate container",
                ))
            })?,
            // If there nothing at that memory address in state, make a new container, and create a new certificate
            None => proto::certificate::CertificateContainer::new(),
        };

        // Use an iterator to find the index of an certificate's ID that matches the certificate attempting to be created
        if let Some((i, _)) = certificates
            .entries
            .iter()
            .enumerate()
            .find(|(_i, certificate)| certificate.id == certificate_id)
        {
            // If that certificate already exists, set certificate_slice to that certificate
            let certificate_slice = certificates.entries.as_mut_slice();
            certificate_slice[i] = certificate;
        } else {
            // Push new and unique certificate to the CertificateContainer
            certificates.entries.push(certificate);
            certificates.entries.sort_by_key(|o| o.clone().id);
        }

        let serialized = protobuf::Message::write_to_bytes(&certificates).map_err(|_err| {
            ApplyError::InvalidTransaction(String::from("Cannot serialize certificate container"))
        })?;

        // Insert serialized CertificateContainer to an address in the merkle tree
        let mut sets = HashMap::new();
        sets.insert(address, serialized);
        self.context.set_state(sets)?;
        Ok(())
    }

    /// As the addressing scheme does not guarantee uniquesness, this adds a Request into a Request Container
    /// which works like a hashbucket, serializes the container and puts it into state,
    /// ```
    /// # Errors
    /// Returns an error if it fails to serialize the container or fails to set it to state
    /// ```
    pub fn set_request(
        &mut self,
        request_id: &str,
        request: proto::request::Request,
    ) -> Result<(), ApplyError> {
        let address = addressing::make_request_address(request_id);
        let state_data = self.context.get_state(vec![address.clone()])?;
        let mut requests: proto::request::RequestContainer = match state_data {
            Some(data) => protobuf::parse_from_bytes(data.as_slice()).map_err(|_err| {
                ApplyError::InvalidTransaction(String::from("Cannot deserialize request container"))
            })?,
            // If there nothing at that memory address in state, make a new container, and create a new request
            None => proto::request::RequestContainer::new(),
        };

        // Use an iterator to find the index of a request_id that matches the request_id attempting to be created
        if let Some((i, _)) = requests
            .entries
            .iter()
            .enumerate()
            .find(|(_i, request)| request.id == request_id)
        {
            // If that request already exists, set requests_slice to that request
            let request_slice = requests.entries.as_mut_slice();
            request_slice[i] = request;
        } else {
            // Push new and unique request to the RequestContainer
            requests.entries.push(request);
            requests.entries.sort_by_key(|a| a.clone().id);
        }

        let serialized = protobuf::Message::write_to_bytes(&requests).map_err(|_err| {
            ApplyError::InvalidTransaction(String::from("Cannot serialize request container"))
        })?;

        // Insert serialized RequestContainer to an address in the merkle tree
        let mut sets = HashMap::new();
        sets.insert(address, serialized);
        self.context.set_state(sets)?;
        Ok(())
    }

    /// As the addressing scheme does not guarantee uniquesness, this adds a Standard into a Standard Container
    /// which works like a hashbucket, serializes the container and puts it into state,
    /// ```
    /// # Errors
    /// Returns an error if it fails to serialize the container or fails to set it to state
    /// ```
    pub fn set_standard(
        &mut self,
        standard_id: &str,
        standard: proto::standard::Standard,
    ) -> Result<(), ApplyError> {
        let address = addressing::make_standard_address(standard_id);
        let state_data = self.context.get_state(vec![address.clone()])?;
        let mut standards: proto::standard::StandardContainer = match state_data {
            Some(data) => protobuf::parse_from_bytes(data.as_slice()).map_err(|_err| {
                ApplyError::InvalidTransaction(String::from(
                    "Cannot deserialize standard container",
                ))
            })?,
            // If there nothing at that memory address in state, make a new container, and create a new standard
            None => proto::standard::StandardContainer::new(),
        };

        // Use an iterator to find the index of a standard_id that matches the standard_id attempting to be created
        if let Some((i, _)) = standards
            .entries
            .iter()
            .enumerate()
            .find(|(_i, standard)| standard.id == standard_id)
        {
            // If that request already exists, set requests_slice to that request
            let standard_slice = standards.entries.as_mut_slice();
            standard_slice[i] = standard;
        } else {
            // Push new and unique request to the StandardContainer
            standards.entries.push(standard);
            standards.entries.sort_by_key(|a| a.clone().id);
        }

        let serialized = protobuf::Message::write_to_bytes(&standards).map_err(|_err| {
            ApplyError::InvalidTransaction(String::from("Cannot serialize standard container"))
        })?;

        // Insert serialized StandardContainer to an address in the merkle tree
        let mut sets = HashMap::new();
        sets.insert(address, serialized);
        self.context.set_state(sets)?;
        Ok(())
    }
}
