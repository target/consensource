// Copyright 2018 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate clap;
extern crate crypto;
extern crate futures;
extern crate hyper;
extern crate protobuf;
extern crate sawtooth_sdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate common;
extern crate serde_json;
extern crate serde_yaml;
extern crate tokio_core;
extern crate users;
extern crate uuid;
extern crate yaml_rust;

mod commands;
mod error;
mod key;
mod submit;
mod transaction;

use clap::ArgMatches;
use error::CliError;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args = parse_args();

    let result = match args.subcommand() {
        ("agent", Some(args)) => commands::agent::run(args),
        ("genesis", Some(args)) => commands::genesis::run(args),
        ("organization", Some(args)) => commands::organization::run(args),
        ("certificate", Some(args)) => commands::certificate::run(args),
        ("standard", Some(args)) => commands::standard::run(args),
        ("accreditation", Some(args)) => commands::accreditation::run(args),
        _ => Err(CliError::InvalidInputError(String::from(
            "Invalid subcommand. Pass --help for usage",
        ))),
    };

    std::process::exit(match result {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("Error: {}", err);
            1
        }
    });
}

fn parse_args<'a>() -> ArgMatches<'a> {
    let app = clap_app!(csrc =>
        (name: APP_NAME)
        (version: VERSION)
        (about: "Consensource CLI")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand agent =>
            (about: "manage the agent")
            (@subcommand create =>
                (about: "create an agent")
                (@arg name: +required "Name of the agent to be created")
                (@arg key: -k --key +takes_value "Signing key name")
                (@arg url: --url +takes_value "URL to the Sawtooth REST API")
            )
            (@subcommand authorize =>
                (about: "authorize an agent")
                (@arg authorize_agent: +required "Pub key of the agent we are authorizing")
                (@arg org_id: +required "Organization agent is associated with")
                (@arg role: +required "Role of the agent: 1 (ADMIN) or 2 (TRANSACTOR)")
                (@arg key: -k --key +takes_value "Signing key of the admin doing the authoriation")
                (@arg url: --url +takes_value "URL to the Sawtooth REST API")
            )
        )

        (@subcommand genesis =>
            (about: "Generate batches in order to bootstrap a genesis block")
            (@arg dry_run: --("dry-run")
             "Processes the input and generates the transactions, but does not generate the output")
            (@arg output: -o --output +takes_value default_value("consensource-genesis.batch")
             "Output file for the resulting batches")
            (@arg descriptor: -g --("genesis-descriptor") +takes_value default_value("genesis.yaml")
             "The genesis descriptor yaml file")
            (@arg keys_directory: -K --("keys-directory") +takes_value
             "An optional directory to write out the keys used when generating the various transactions"))

        (@subcommand organization =>
            (about: "manage the organization")
            (@subcommand create =>
                (about: "create an organization")
                (@arg name: +required "Name of the organization to be created")
                (@arg org_type: +required "Type of the organization to be created:
                1 (CERTIFYING_BODY), 2 (STANDARDS_BODY), or 3 (FACTORY)")
                (@arg contact_name: +required "Name of the organization's contact")
                (@arg contact_phone_number: +required "Phone number of the organization's contact")
                (@arg contact_language_code: +required "Language of the organization's contact")
                (@arg street_address: --street_address +takes_value "Street address of the organization's contact")
                (@arg city: --city +takes_value "City of the factory")
                (@arg country: --country +takes_value "Country of the factory")
                (@arg key: -k --key +takes_value "Signing key name")
                (@arg url: --url +takes_value "URL to the Sawtooth REST API")
            )
        )
        (@subcommand certificate =>
            (about: "manage the certificate")
            (@subcommand create =>
                (about: "issue a certificate")
                (@arg id: +required "Id of the certificate to be issued")
                (@arg certifying_body_id: +required "Certifying body that is issuing the certificate")
                (@arg factory_id: "Factory the certificate is being issued to")
                (@arg source: +required "The source that triggered the IssueCertificate Trasaction:
                1 (FROM_REQUEST): it means the IssueCertificateAction is associated to a request made by a factory.
                The argument request_id must be passed as well.
                2 (INDEPENDENT):  it means the IssueCertificateAction is not associated with a request made by a factory.
                The field factory_name must passed as well")
                (@arg request_id: --request_id +takes_value "Id of the certificate request made by the factory")
                (@arg standard_id: "Standard that this certificate is for")
                (@arg cert_data: -cd --cert_data +takes_value +multiple "Optional cert data")
                (@arg valid_from: +required "Start timestamp of the certificate")
                (@arg valid_to: +required "End timestamp of the certificate")
                (@arg key: -k --key +takes_value "Signing key name")
                (@arg url: --url +takes_value "URL to the Sawtooth REST API")
            )
        )
        (@subcommand standard =>
            (about: "manage standards")
            (@subcommand create =>
                (about: "create a new standard")
                (@arg name: +required "Name of the standard")
                (@arg version: +required "Current version of the standard.")
                (@arg description: +required "Short description of the standard")
                (@arg link: +required "Link to the standard's documentation.")
                (@arg organization_id: +required "Id of the organization creating the standard")
                (@arg approval_date: +required "Date the standard is officially issued. Format: seconds since Unix epoch")
                (@arg key: -k --key +takes_value "Signing key name")
                (@arg url: --url +takes_value "URL to the Sawtooth REST API")
            )
        )
        (@subcommand accreditation =>
            (about: "manage accreditations")
            (@subcommand create =>
                (about: "accredit an certifying body to an standard")
                (@arg certifying_body_id: +required "Id of the certifying body that is being accredited.")
                (@arg standards_body_id: +required "Id of the standards body that is issuing the accreditation.")
                (@arg standard_id: +required "Id of the standard that the certifying body is being accredited for.")
                (@arg valid_from: +required "Time the accreditation was issued. Format: seconds since Unix epoch")
                (@arg valid_to: +required "When the accreditation will become invalid. Format: seconds since Unix epoch")
                (@arg key: -k --key +takes_value "Signing key name")
                (@arg url: --url +takes_value "URL to the Sawtooth REST API")
            )
        )
    );
    app.get_matches()
}
