rest-api tested for open source using: rustc 1.31.0-nightly (1cf82fd9c 2018-10-30)

the rest of the project should run on the stable toolchain

# ConsenSource

ConsenSource is a blockchain application to help verify that products are sourced sustainably from certified factories. Specifically, this application serves as a common platform to verify and display supplier certifications and audit data between standards bodies, certification bodies, and factories. This application runs on Hyperledger Sawtooth, an enterprise blockchain platform.

The ConsenSource repository includes several components:

- A transaction processor to handle the certification registry's transaction logic

- A custom REST API that provides endpoints for accessing blockchain data, user information, and state data

- An event subscriber that listens to blockchain events in order to parse this data to store this data in an off-chain reporting database

- Multiple client web apps to give a sample of the interactions each entity may have with the ConsenSource application, including standards bodies, certification bodies, factories and retailers

- A command line interface with basic initial commands to create state objects, including agents, organizations, certificates, standards and accreditations


## Usage

This application runs using separate Docker containers for the various components. These Docker images may be run together using the `docker-compose.yaml` file included within the repository.

To run the ConsenSource application, run the following command in the project's root directory:

`docker-compose -f docker-compose.yaml up`


Instructions on how to build, run and develop the web client can be found in the [README](https://github.com/target/ConsenSource/blob/master/client/README.md) in the client sub-directory.

Each client application provides functionality depending on the entity, based on their unique interactions with the common platform. The following section provides the default URL for each client application and their associated functionality:

- Retailer Client: http://localhost:8080/
  - Serves as an example to information available to Target sourcing members
  - View factories and their associated contact, location, and certification information
  - View agents and, if applicable, associated organization and contact information

- Factory Client: http://localhost:8080/index_factory.html
  - Open certification requests
  - View history of granted certifications
  - Update any address or contact information

- Standards Body Client: http://localhost:8080/index_standards_body.html
  - View, create, and update standards
  - Accredit certifying bodies to issue certificates

- Certifying Body Client: http://localhost:8080/index_auditor.html
  - View factories' certification requests
  - Issue certifications based on these requests
  - View all factories

Other available endpoints include:

- PostgreSQL Adminer: http://localhost:8081
  - Login credentials:
    - System: PostgreSQL
    - Server: postgres:5432
    - Username: cert-registry
    - Password: cert-registry
    - Database: cert-registry

Refer to the [REST API specification document](https://github.com/target/ConsenSource/blob/master/docs_content/rest-api/specs.yaml) for further information on available endpoints


## Testing

Docker based integration tests are available for the application. In order to execute the tests, run the following command in the project's root directory:

`./bin/run-tests`

This will build and run each test suite in the `integration_tests` directory. In order to run a specific test suite, pass the name of the integration tests directory as an argument to the script. For example, to run only the processor tests, run the following command:

`./bin/run-tests processor`

The script also provides a `--no-build` option, which tells the script not to rebuild the images or recompile the component code before running the tests. This can be useful to save time when writing new tests if no changes have been made to the component. For example, to run the processor tests without recompiling code or rebuilding images, run the following command:

`./bin/run-tests --no-build processor`


## Further information

For further information on ConsenSource, including more details of the components, glossary and FAQs are available in the [ConsenSource Docs](https://pages.github.com/target/ConsenSource/)

## Authors

- [Adeeb Ahmed](https://github.com/adeebahmed)
- [Brijhette Farmer](https://github.com/astrohfiziks)
- [Darian Plumb](https://github.com/dplumb94)
- [Eloa Franca Verona](https://github.com/eloaverona)
- [Emmanuel Meinen](https://github.com/meinenec)
- [Peter Schwarz](https://github.com/peterschwarz)
- [Raphael Santo Domingo](https://github.com/pa3ng)
- [Sean Quinn](https://github.com/sjqnn)
- [Shannyn Telander](https://github.com/shannynalayna)
