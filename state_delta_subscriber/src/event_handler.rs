use common::addressing::{get_address_type, get_family_namespace_prefix, AddressSpace};
use common::proto::{agent, certificate, organization, request, standard};
use database::{
    custom_types::*,
    data_manager::{DataManager, OperationType, MAX_BLOCK_NUM},
    models::*,
};
use protobuf;
use regex::Regex;
use sawtooth_sdk::messages::events::{Event, EventList, Event_Attribute};
use sawtooth_sdk::messages::transaction_receipt::{StateChange, StateChangeList};

use transformer::{Container, FromStateAtBlock};

use errors::SubscriberError;

/// Given a connection to the reporting database, it parses the event data received from the
/// subscriber and adds that data to reporting DB.
pub struct EventHandler {
    data_manager: DataManager,
}

impl EventHandler {
    pub fn new(data_manager: DataManager) -> EventHandler {
        EventHandler { data_manager }
    }

    pub fn handle_events(&self, data: &[u8]) -> Result<(), SubscriberError> {
        let (block, operations) = self.parse_events(data)?;
        self.data_manager
            .execute_operations_in_block(operations, &block)?;
        info!("Successfully submited event data to reporting database");
        Ok(())
    }

    fn parse_events(&self, data: &[u8]) -> Result<(Block, Vec<OperationType>), SubscriberError> {
        let event_list: EventList = Self::unpack_data(data);
        let events = event_list.get_events().to_vec();
        let block = self.parse_block(&events)?;
        let state_changes = self.parse_state_delta_events(&events)?;
        let mut operations = Vec::<OperationType>::new();
        for change in state_changes {
            operations.push(self.parse_operation(&change, &block)?);
        }
        Ok((block, operations))
    }

    fn parse_block(&self, events: &[Event]) -> Result<Block, SubscriberError> {
        events
            .into_iter()
            .filter(|e| e.get_event_type() == "sawtooth/block-commit")
            .map(|block_commit_event| {
                let block_num: Vec<Event_Attribute> = block_commit_event
                    .get_attributes()
                    .to_vec()
                    .into_iter()
                    .filter(|a| a.get_key() == "block_num")
                    .collect();
                let block_id: Vec<Event_Attribute> = block_commit_event
                    .get_attributes()
                    .to_vec()
                    .into_iter()
                    .filter(|a| a.get_key() == "block_id")
                    .collect();

                Ok(Block {
                    block_num: block_num[0]
                        .get_value()
                        .parse::<i64>()
                        .map_err(|err| SubscriberError::EventParseError(err.to_string()))?,
                    block_id: block_id[0].get_value().to_string(),
                })
            })
            .last()
            .unwrap_or_else(|| {
                Err(SubscriberError::EventParseError(
                    "No block event found".to_string(),
                ))
            })
    }

    fn parse_state_delta_events(
        &self,
        events: &[Event],
    ) -> Result<Vec<StateChange>, SubscriberError> {
        let namespace_regex = self.get_namespace_regex();
        Ok(events
            .into_iter()
            .filter(|e| e.get_event_type() == "sawtooth/state-delta")
            .flat_map(|event| {
                let mut change_list = Self::unpack_data::<StateChangeList>(event.get_data());
                change_list
                    .take_state_changes()
                    .into_iter()
                    .filter(|state_change| (namespace_regex.is_match(state_change.get_address())))
            })
            .collect())
    }

    fn get_namespace_regex(&self) -> Regex {
        let namespace = get_family_namespace_prefix();
        Regex::new(&format!(r"^{}", namespace)).unwrap()
    }

    /// Deserializes binary data to a protobuf message
    fn unpack_data<T>(data: &[u8]) -> T
    where
        T: protobuf::Message,
    {
        protobuf::parse_from_bytes(&data).expect("Error parsing protobuf data.")
    }

    /// Given a state change it deserializes the data to a protobuf message,
    /// and converts that message into objects that can be inserted in the
    /// database via the data_manager.
    /// ```
    /// # Errors
    /// Returns an error if State Change address is not part of the Certificate Registry Namespace
    /// ```
    fn parse_operation(
        &self,
        state: &StateChange,
        block: &Block,
    ) -> Result<OperationType, SubscriberError> {
        let address_type = get_address_type(state.get_address());
        match address_type {
            AddressSpace::Organization => {
                let org_container: organization::OrganizationContainer =
                    Self::unpack_data(state.get_value());

                let transaction =
                    OperationType::CreateOrganization(org_container.to_models(block.block_num));
                Ok(transaction)
            }
            AddressSpace::Agent => {
                let agent_container: agent::AgentContainer = Self::unpack_data(state.get_value());
                let transaction =
                    OperationType::CreateAgent(agent_container.to_models(block.block_num));
                Ok(transaction)
            }
            AddressSpace::Certificate => {
                let cert_container: certificate::CertificateContainer =
                    Self::unpack_data(state.get_value());
                let transaction =
                    OperationType::CreateCertificate(cert_container.to_models(block.block_num));
                Ok(transaction)
            }
            AddressSpace::Request => {
                let request_container: request::RequestContainer =
                    Self::unpack_data(state.get_value());
                let transaction =
                    OperationType::CreateRequest(request_container.to_models(block.block_num));
                Ok(transaction)
            }
            AddressSpace::Standard => {
                let standard_container: standard::StandardContainer =
                    Self::unpack_data(state.get_value());
                let transaction =
                    OperationType::CreateStandard(standard_container.to_models(block.block_num));
                Ok(transaction)
            }
            AddressSpace::AnotherFamily => Err(SubscriberError::EventParseError(
                "Address didnt match any existent state data
                types in the Certificate Registry Namespace."
                    .to_string(),
            )),
        }
    }
}

containerize!(
    organization::Organization,
    organization::OrganizationContainer
);
impl FromStateAtBlock<organization::Organization>
    for (
        NewOrganization,
        Option<Vec<NewAccreditation>>,
        Option<NewAddress>,
        Vec<NewAuthorization>,
        Vec<NewContact>,
    )
{
    fn at_block(block_num: i64, org: &organization::Organization) -> Self {
        let new_org = NewOrganization {
            organization_id: org.id.clone(),
            name: org.name.clone(),
            organization_type: match org.organization_type {
                organization::Organization_Type::CERTIFYING_BODY => {
                    OrganizationTypeEnum::CertifyingBody
                }
                organization::Organization_Type::STANDARDS_BODY => {
                    OrganizationTypeEnum::StandardsBody
                }
                organization::Organization_Type::FACTORY => OrganizationTypeEnum::Factory,
                organization::Organization_Type::UNSET_TYPE => OrganizationTypeEnum::UnsetType,
            },
            start_block_num: block_num,
            end_block_num: MAX_BLOCK_NUM,
        };
        let new_accreditations = match org.get_organization_type() {
            organization::Organization_Type::CERTIFYING_BODY => {
                let accreditations: Vec<NewAccreditation> = org
                    .get_certifying_body_details()
                    .clone()
                    .accreditations
                    .iter()
                    .map(|accreditation| NewAccreditation {
                        organization_id: org.id.clone(),
                        standard_id: accreditation.get_standard_id().to_string(),
                        standard_version: accreditation.get_standard_version().to_string(),
                        accreditor_id: accreditation.get_accreditor_id().to_string(),
                        valid_from: accreditation.get_valid_from() as i64,
                        valid_to: accreditation.get_valid_to() as i64,
                        start_block_num: block_num,
                        end_block_num: MAX_BLOCK_NUM,
                    })
                    .collect();
                Some(accreditations)
            }
            _ => None,
        };
        let new_auths = org
            .authorizations
            .iter()
            .map(|auth| NewAuthorization {
                organization_id: org.id.clone(),
                public_key: auth.get_public_key().to_string(),
                role: match auth.get_role() {
                    organization::Organization_Authorization_Role::ADMIN => RoleEnum::Admin,
                    organization::Organization_Authorization_Role::TRANSACTOR => {
                        RoleEnum::Transactor
                    }
                    organization::Organization_Authorization_Role::UNSET_ROLE => {
                        RoleEnum::UnsetRole
                    }
                },
                start_block_num: block_num,
                end_block_num: MAX_BLOCK_NUM,
            })
            .collect();
        let new_contacts = org
            .contacts
            .iter()
            .map(|contact| NewContact {
                organization_id: org.id.clone(),
                name: contact.get_name().to_string(),
                phone_number: contact.get_phone_number().to_string(),
                language_code: contact.get_language_code().to_string(),
                start_block_num: block_num,
                end_block_num: MAX_BLOCK_NUM,
            })
            .collect();
        let new_address = match org.get_organization_type() {
            organization::Organization_Type::FACTORY => {
                let address = org
                    .get_factory_details()
                    .clone()
                    .address
                    .map(|address| NewAddress {
                        organization_id: org.id.clone(),
                        street_line_1: address.get_street_line_1().to_string(),
                        street_line_2: match address.get_street_line_2() {
                            "" => None,
                            _ => Some(address.get_street_line_2().to_string()),
                        },
                        city: address.get_city().to_string(),
                        state_province: match address.get_state_province() {
                            "" => None,
                            _ => Some(address.get_state_province().to_string()),
                        },
                        country: address.get_country().to_string(),
                        postal_code: match address.get_postal_code() {
                            "" => None,
                            _ => Some(address.get_postal_code().to_string()),
                        },
                        start_block_num: block_num,
                        end_block_num: MAX_BLOCK_NUM,
                    });
                Some(address.unwrap())
            }
            _ => None,
        };

        (
            new_org,
            new_accreditations,
            new_address,
            new_auths,
            new_contacts,
        )
    }
}

containerize!(agent::Agent, agent::AgentContainer);
impl FromStateAtBlock<agent::Agent> for NewAgent {
    fn at_block(block_num: i64, agent: &agent::Agent) -> Self {
        NewAgent {
            public_key: agent.get_public_key().to_string(),
            organization_id: match agent.get_organization_id() {
                "" => None,
                _ => Some(agent.get_organization_id().to_string()),
            },
            name: agent.get_name().to_string(),
            timestamp: agent.get_timestamp() as i64,
            start_block_num: block_num,
            end_block_num: MAX_BLOCK_NUM,
        }
    }
}

containerize!(certificate::Certificate, certificate::CertificateContainer);
impl FromStateAtBlock<certificate::Certificate> for NewCertificate {
    fn at_block(block_num: i64, certificate: &certificate::Certificate) -> Self {
        NewCertificate {
            certificate_id: certificate.get_id().to_string(),
            certifying_body_id: certificate.get_certifying_body_id().to_string(),
            factory_id: certificate.get_factory_id().to_string(),
            standard_id: certificate.get_standard_id().to_string(),
            standard_version: certificate.get_standard_version().to_string(),
            valid_from: certificate.get_valid_from() as i64,
            valid_to: certificate.get_valid_to() as i64,
            start_block_num: block_num,
            end_block_num: MAX_BLOCK_NUM,
        }
    }
}

containerize!(request::Request, request::RequestContainer);
impl FromStateAtBlock<request::Request> for NewRequest {
    fn at_block(block_num: i64, request: &request::Request) -> Self {
        NewRequest {
            request_id: request.get_id().to_string(),
            factory_id: request.get_factory_id().to_string(),
            standard_id: request.get_standard_id().to_string(),
            status: match request.get_status() {
                request::Request_Status::OPEN => RequestStatusEnum::Open,
                request::Request_Status::IN_PROGRESS => RequestStatusEnum::InProgress,
                request::Request_Status::CLOSED => RequestStatusEnum::Closed,
                request::Request_Status::CERTIFIED => RequestStatusEnum::Certified,
                request::Request_Status::UNSET_STATUS => RequestStatusEnum::UnsetStatus,
            },
            request_date: request.get_request_date() as i64,
            start_block_num: block_num,
            end_block_num: MAX_BLOCK_NUM,
        }
    }
}

containerize!(standard::Standard, standard::StandardContainer);
impl FromStateAtBlock<standard::Standard> for (NewStandard, Vec<NewStandardVersion>) {
    fn at_block(block_num: i64, standard: &standard::Standard) -> Self {
        let db_standard = NewStandard {
            standard_id: standard.id.clone(),
            organization_id: standard.organization_id.clone(),
            name: standard.name.clone(),
            start_block_num: block_num,
            end_block_num: MAX_BLOCK_NUM,
        };

        let db_versions = standard
            .versions
            .iter()
            .map(|version| NewStandardVersion {
                standard_id: standard.id.clone(),
                version: version.version.clone(),
                link: version.link.clone(),
                description: version.description.clone(),
                approval_date: version.approval_date as i64,
                start_block_num: block_num,
                end_block_num: MAX_BLOCK_NUM,
            })
            .collect();

        (db_standard, db_versions)
    }
}
