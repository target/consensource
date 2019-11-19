use clap::ArgMatches;

use common::addressing;
use common::proto::payload::CreateOrganizationAction;

use common::proto::payload::{CertificateRegistryPayload, CertificateRegistryPayload_Action};
use error::CliError;
use transaction::{create_batch, create_batch_list_from_one, create_transaction};

use key;
use sawtooth_sdk::signing;
use submit;
use uuid::Uuid;

use common::proto::organization::Factory_Address;
use common::proto::organization::Organization_Contact;
use common::proto::organization::Organization_Type;
use std::{thread, time};

use protobuf;

pub fn run<'a>(args: &ArgMatches<'a>) -> Result<(), CliError> {
    match args.subcommand() {
        ("create", Some(args)) => run_create_command(args),
        _ => Err(CliError::InvalidInputError(String::from(
            "Invalid subcommand. Pass --help for usage",
        ))),
    }
}

fn run_create_command<'a>(args: &ArgMatches<'a>) -> Result<(), CliError> {
    let name = args.value_of("name").unwrap();
    let contact_name = args.value_of("contact_name").unwrap();
    let contact_phone_number = args.value_of("contact_phone_number").unwrap();
    let contact_language_code = args.value_of("contact_language_code").unwrap();
    let street = args.value_of("street_address");
    let city = args.value_of("city");
    let country = args.value_of("country");
    let key = args.value_of("key");
    let url = args.value_of("url").unwrap_or("http://localhost:9009");

    let valid_org_types = "1 - CERTIFYING_BODY \n 2 - STANDARDS_BODY \n 3 - FACTORY";

    let organization_type = match args.value_of("org_type").unwrap() {
        "1" => Ok(Organization_Type::CERTIFYING_BODY),
        "2" => Ok(Organization_Type::STANDARDS_BODY),
        "3" => Ok(Organization_Type::FACTORY),
        "4" => Err(CliError::InvalidInputError(format!(
            "Organization type 4 is not yet implemented. Valid types are: \n {org_types}",
            org_types = valid_org_types
        ))),
        other => Err(CliError::UserError(format!(
            "Invalid organization type: {:?}. Valid types are: \n {org_types}",
            other,
            org_types = valid_org_types
        ))),
    }?;

    if organization_type == Organization_Type::FACTORY {
        match street {
            None => Err(CliError::InvalidInputError(format!(
                "A street address is required for a factory"
            ))),
            other => Ok(other),
        }?;
        match city {
            None => Err(CliError::InvalidInputError(format!(
                "A city is required for a factory"
            ))),
            other => Ok(other),
        }?;
        match country {
            None => Err(CliError::InvalidInputError(format!(
                "A country is required for a factory"
            ))),
            other => Ok(other),
        }?;
    }

    let org_id = Uuid::new_v4().to_string();

    let private_key = key::load_signing_key(key)?;
    let context = signing::create_context("secp256k1")?;
    let factory = signing::CryptoFactory::new(&*context);
    let signer = factory.new_signer(&private_key);

    let payload = create_organization_payload(
        &org_id,
        &name,
        organization_type,
        contact_name,
        contact_phone_number,
        contact_language_code,
        street,
        city,
        country,
    );

    let header_input =
        create_organization_transaction_addresses(&signer.get_public_key()?.as_hex(), &org_id);
    let header_output = header_input.clone();

    let txn = create_transaction(&payload, &signer, header_input, header_output)?;
    let batch = create_batch(txn, &signer)?;
    let batch_list = create_batch_list_from_one(batch);

    let mut org_status = submit::submit_batch_list(url, &batch_list)
        .and_then(|link| submit::wait_for_status(url, &link))?;

    loop {
        match org_status
            .data
            .get(0)
            .expect("Expected a batch status, but was not found")
            .status
            .as_ref()
        {
            "COMMITTED" => break Ok(()),
            "INVALID" => {
                break Err(CliError::InvalidTransactionError(
                    org_status.data[0]
                        .invalid_transactions
                        .get(0)
                        .expect("Expected a transaction status, but was not found")
                        .message
                        .clone(),
                ));
            }
            // "PENDING" case where we should recheck
            _ => {
                thread::sleep(time::Duration::from_millis(3000));
                org_status = submit::wait_for_status(url, &org_status.link)?;
            }
        }
    }
}

pub fn create_organization_payload(
    id: &str,
    name: &str,
    org_type: Organization_Type,
    contact_name: &str,
    contact_phone_number: &str,
    contact_language_code: &str,
    street: Option<&str>,
    city: Option<&str>,
    country: Option<&str>,
) -> CertificateRegistryPayload {
    let mut organization = CreateOrganizationAction::new();
    organization.set_name(String::from(name));
    organization.set_id(String::from(id));
    organization.set_organization_type(org_type);

    if org_type == Organization_Type::FACTORY {
        let mut factory_address = Factory_Address::new();
        factory_address.set_street_line_1(street.unwrap().to_string());
        factory_address.set_city(city.unwrap().to_string());
        factory_address.set_country(country.unwrap().to_string());
        organization.set_address(factory_address);
    }

    let mut contact = Organization_Contact::new();
    contact.set_name(String::from(contact_name));
    contact.set_phone_number(String::from(contact_phone_number));
    contact.set_language_code(String::from(contact_language_code));
    organization.set_contacts(protobuf::RepeatedField::from_vec(vec![contact]));

    let mut payload = CertificateRegistryPayload::new();
    payload.action = CertificateRegistryPayload_Action::CREATE_ORGANIZATION;
    payload.set_create_organization(organization);
    payload
}

pub fn create_organization_transaction_addresses(
    public_key: &str,
    organization_id: &str,
) -> Vec<String> {
    let agent_address = addressing::make_agent_address(public_key);
    let org_address = addressing::make_organization_address(organization_id);
    vec![agent_address, org_address]
}
