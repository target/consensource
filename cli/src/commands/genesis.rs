use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::NaiveDate;
use clap::ArgMatches;
use common::proto::organization::Organization_Type;
use protobuf;
use protobuf::Message;
use sawtooth_sdk::messages::batch::Batch;
use sawtooth_sdk::messages::batch::BatchList;
use sawtooth_sdk::signing;
use serde;
use serde_yaml;
use uuid::Uuid;

use commands::agent::{create_agent_payload, create_agent_transaction_addresses};
use commands::organization::{
    create_organization_payload, create_organization_transaction_addresses,
};
use commands::standard::{create_standard_payload, create_standard_transaction_addresses};
use error::CliError;
use transaction::{create_batch, create_transaction};

#[derive(Serialize, Deserialize, Debug)]
struct GenesisAgent {
    email: String,
    organization: Option<GenesisOrganization>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum GenesisOrganization {
    StandardsBody {
        name: String,
        contact: GenesisContact,
        standards: Vec<GenesisStandard>,
    },
    CertifyingBody {
        name: String,
        contact: GenesisContact,
    },
    Factory {
        name: String,
        contact: GenesisContact,
        address: GenesisAddress,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct GenesisStandard {
    name: String,
    version: String,
    description: String,
    link: String,
    #[serde(deserialize_with = "date_to_epoch_time")]
    approval_date: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct GenesisContact {
    name: String,
    phone_number: String,
    language: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GenesisAddress {
    street_1: String,
    street_2: Option<String>,
    city: String,
    state_province: Option<String>,
    postal_code: Option<String>,
    country: String,
}

pub fn run<'a>(args: &ArgMatches<'a>) -> Result<(), CliError> {
    let context = signing::create_context("secp256k1")?;
    let factory = signing::CryptoFactory::new(&*context);

    let output_file = args
        .value_of("output")
        .unwrap_or("consensource-genesis.batch");
    let generated_keys_dir = args.value_of("keys_directory");
    let genesis_descriptor = args.value_of("descriptor").unwrap_or("genesis.yaml");

    let descriptor_file = File::open(&Path::new(genesis_descriptor))?;

    let agents: Vec<GenesisAgent> = serde_yaml::from_reader(descriptor_file).map_err(|err| {
        CliError::InvalidInputError(format!("Unable to parse genesis descriptor: {:?}", err))
    })?;

    let mut batches = vec![];

    for agent in agents {
        let private_key = context.new_random_private_key()?;
        let signer = factory.new_signer(&*private_key);

        let create_time = current_epoch_time();
        let payload = create_agent_payload(&agent.email, create_time);

        let header_input = create_agent_transaction_addresses(&signer.get_public_key()?.as_hex());
        let header_output = header_input.clone();
        let txn = create_transaction(&payload, &signer, header_input, header_output)?;
        let batch = create_batch(txn, &signer)?;
        batches.push(batch);

        if let Some(org) = agent.organization {
            let mut org_batches = create_org_batches(&signer, &org)?;
            batches.append(&mut org_batches);
        }

        if let Some(key_dir) = generated_keys_dir {
            store_key(&signer, &*private_key, &agent.email, key_dir)?;
        }
    }

    let mut batch_list = BatchList::new();
    batch_list.set_batches(protobuf::RepeatedField::from_vec(batches));

    if !args.is_present("dry_run") {
        let mut out = File::create(&Path::new(output_file))?;
        batch_list.write_to_writer(&mut out)?;
    }

    Ok(())
}

fn create_org_batches<'s>(
    signer: &'s signing::Signer,
    org: &GenesisOrganization,
) -> Result<Vec<Batch>, CliError> {
    let mut batches = vec![];
    let org_id = Uuid::new_v4().to_string();
    let (name, organization_type, contact, address, standards) = match org {
        GenesisOrganization::StandardsBody {
            name,
            contact,
            standards,
            ..
        } => (
            name,
            Organization_Type::STANDARDS_BODY,
            contact,
            None,
            Some(standards),
        ),
        GenesisOrganization::CertifyingBody { name, contact, .. } => (
            name,
            Organization_Type::CERTIFYING_BODY,
            contact,
            None,
            None,
        ),
        GenesisOrganization::Factory {
            name,
            contact,
            address,
        } => (
            name,
            Organization_Type::FACTORY,
            contact,
            Some(address),
            None,
        ),
    };

    let payload = create_organization_payload(
        &org_id,
        &name,
        organization_type,
        &contact.name,
        &contact.phone_number,
        &contact.language,
        address.as_ref().map(|a| &*a.street_1),
        address.as_ref().map(|a| &*a.city.as_str()),
        address.as_ref().map(|a| &*a.country.as_str()),
    );

    let header_input =
        create_organization_transaction_addresses(&signer.get_public_key()?.as_hex(), &org_id);
    let header_output = header_input.clone();

    let txn = create_transaction(&payload, &signer, header_input, header_output)?;
    batches.push(create_batch(txn, &signer)?);

    if let Some(standards) = standards {
        for standard in standards {
            let payload = create_standard_payload(
                &standard.name,
                &standard.version,
                &standard.description,
                &standard.link,
                standard.approval_date,
            );
            let (inputs, outputs) = create_standard_transaction_addresses(
                &signer,
                payload.get_create_standard().get_standard_id(),
                &org_id,
            )?;
            let txn = create_transaction(&payload, &signer, inputs, outputs)?;
            batches.push(create_batch(txn, &signer)?);
        }
    }

    Ok(batches)
}

fn store_key(
    signer: &signing::Signer,
    private_key: &dyn signing::PrivateKey,
    user_identifier: &str,
    key_dir: &str,
) -> Result<(), CliError> {
    let pub_key_hex = signer.get_public_key()?.as_hex();
    let priv_key_hex = private_key.as_hex();

    let mut pub_key_file = PathBuf::new();
    pub_key_file.push(key_dir);
    pub_key_file.push(format!("{}.pub", user_identifier));

    File::create(&pub_key_file)?.write_all(pub_key_hex.as_bytes())?;

    let mut priv_key_file = PathBuf::new();
    priv_key_file.push(key_dir);
    priv_key_file.push(format!("{}.priv", user_identifier));

    File::create(&priv_key_file)?.write_all(priv_key_hex.as_bytes())?;

    Ok(())
}

fn current_epoch_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_secs()
}

fn date_to_epoch_time<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(DateVisitor)
}

struct DateVisitor;

impl<'de> serde::de::Visitor<'de> for DateVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a date formatted as YYYY/MM/DD")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match NaiveDate::parse_from_str(s, "%Y/%m/%d") {
            Ok(date) => Ok(date.and_hms(0, 0, 0).timestamp() as u64),
            Err(_) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(s),
                &self,
            )),
        }
    }
}
