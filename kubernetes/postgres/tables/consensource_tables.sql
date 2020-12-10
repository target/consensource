-------------------------------
-- ConsenSource database tables
-------------------------------
----------------------------------------------------------------------------------------------
-- This file is for running a local postgres database on the docker-desktop cluster
-- specifically for a local ConsenSource deployment.
--
-- Make sure the section of the file below is up-to-date with
-- https://github.com/target/consensource-database/blob/master/tables/cert_registry_tables.sql
-- 
----------------------------------------------------------------------------------------------


-- Add pg_trgm extension for similarity index searches
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Create custom types

CREATE TYPE Role AS ENUM ('ADMIN', 'TRANSACTOR', 'UNSET_ROLE');
CREATE TYPE OrganizationType AS ENUM ('STANDARDS_BODY', 'CERTIFYING_BODY', 'FACTORY', 'INGESTION', 'UNSET_TYPE');
CREATE TYPE RequestStatus AS ENUM ('OPEN', 'IN_PROGRESS', 'CLOSED', 'CERTIFIED', 'UNSET_STATUS');
CREATE TYPE AssertionType AS ENUM ('STANDARD', 'CERTIFICATE', 'FACTORY', 'UNSET_TYPE');

-- Create tables

CREATE TABLE blocks (
  block_num     BIGINT    PRIMARY KEY,
  block_id      VARCHAR   NOT NULL
);


CREATE TABLE IF NOT EXISTS chain_record (
  id                         BIGSERIAL  PRIMARY KEY,
  start_block_num            BIGINT     NOT NULL,
  end_block_num              BIGINT     NOT NULL
);


CREATE TABLE IF NOT EXISTS agents (
  id                         BIGSERIAL  PRIMARY KEY,
  public_key                 VARCHAR    NOT NULL,
  name                       VARCHAR    NOT NULL,
  organization_id            VARCHAR,
  timestamp                  BIGINT     NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS agents_pub_key_index ON agents (public_key);
CREATE INDEX IF NOT EXISTS agents_block_index ON agents (end_block_num);

CREATE TABLE IF NOT EXISTS organizations (
  id                         BIGSERIAL  PRIMARY KEY,
  organization_id            VARCHAR    NOT NULL,
  name                       VARCHAR    NOT NULL,
  organization_type          OrganizationType  NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS organizations_organization_id_index ON organizations (organization_id);
CREATE INDEX IF NOT EXISTS organizations_block_index ON organizations (end_block_num);

CREATE TABLE IF NOT EXISTS contacts (
  id                         BIGSERIAL  PRIMARY KEY,
  organization_id            VARCHAR    NOT NULL,
  name                       VARCHAR    NOT NULL,
  phone_number               VARCHAR    NOT NULL,
  language_code              VARCHAR    NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS contacts_organization_id_index ON contacts (organization_id);
CREATE INDEX IF NOT EXISTS contacts_block_index ON contacts (end_block_num);

CREATE TABLE IF NOT EXISTS authorizations (
  id                         BIGSERIAL  PRIMARY KEY,
  organization_id            VARCHAR    NOT NULL,
  public_key                 VARCHAR    NOT NULL,
  role                       Role       NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS authorizations_organization_id_index ON authorizations (organization_id);
CREATE INDEX IF NOT EXISTS authorizations_public_key_index ON authorizations (public_key);
CREATE INDEX IF NOT EXISTS authorizations_block_index ON authorizations (end_block_num);

CREATE TABLE IF NOT EXISTS accreditations (
    id                      BIGSERIAL   PRIMARY KEY,
    organization_id         VARCHAR     NOT NULL,
    standard_id             VARCHAR     NOT NULL,
    standard_version        VARCHAR     NOT NULL,
    accreditor_id           VARCHAR     NOT NULL,
    valid_from              BIGINT      NOT NULL,
    valid_to                BIGINT      NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS accreditations_organization_id_index ON accreditations (organization_id);
CREATE INDEX IF NOT EXISTS accreditations_block_index ON accreditations (end_block_num);

CREATE TABLE IF NOT EXISTS certificates (
  id                         BIGSERIAL  PRIMARY KEY,
  certificate_id             VARCHAR    NOT NULL,
  certifying_body_id         VARCHAR    NOT NULL,
  factory_id                 VARCHAR    NOT NULL,
  standard_id                VARCHAR    NOT NULL,
  standard_version           VARCHAR    NOT NULL,
  valid_from                 BIGINT     NOT NULL,
  valid_to                   BIGINT     NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS certificates_certificate_id_index ON certificates (certificate_id);
CREATE INDEX IF NOT EXISTS certificates_block_index ON certificates (end_block_num);

CREATE TABLE IF NOT EXISTS addresses (
  id                 BIGSERIAL   PRIMARY KEY,
  organization_id    VARCHAR     NOT NULL,
  street_line_1      VARCHAR     NOT NULL,
  street_line_2      VARCHAR,
  city               VARCHAR     NOT NULL,
  state_province     VARCHAR,
  country            VARCHAR     NOT NULL,
  postal_code        VARCHAR,
  text_searchable_address_col   TSVECTOR,
  full_address       VARCHAR
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS addresses_organization_id_index ON addresses (organization_id);
CREATE INDEX IF NOT EXISTS addresses_block_index ON addresses (end_block_num);
CREATE INDEX IF NOT EXISTS address_text_search ON addresses USING GIN (text_searchable_address_col);
CREATE INDEX addresses_city_trgm_idx ON addresses USING GIST (city gist_trgm_ops);
CREATE INDEX addresses_state_trgm_idx ON addresses USING GIST (state_province gist_trgm_ops);
CREATE INDEX addresses_country_trgm_idx ON addresses USING GIST (country gist_trgm_ops);
CREATE INDEX addresses_full_address_idx ON addresses USING GIST (full_address gist_trgm_ops);

CREATE TRIGGER tsvectorupdateaddresses BEFORE INSERT OR UPDATE
ON addresses FOR EACH ROW EXECUTE PROCEDURE
tsvector_update_trigger(text_searchable_address_col, 'pg_catalog.english',
  street_line_1, street_line_2, city, state_province, country, postal_code);

CREATE OR REPLACE FUNCTION full_address_update_trigger() RETURNS TRIGGER AS $full_address_update_trigger$
  BEGIN
    NEW.full_address = NEW.city || ' ' || NEW.state_province || ' ' || NEW.country || ' ' || NEW.postal_code;
    RETURN NEW;
  END;
$full_address_update_trigger$ LANGUAGE PLPGSQL;

CREATE TRIGGER updatefulladdresses BEFORE INSERT OR UPDATE
ON addresses FOR EACH ROW EXECUTE PROCEDURE
full_address_update_trigger();

CREATE TABLE IF NOT EXISTS certificate_data (
  id                         BIGSERIAL   PRIMARY KEY,
  certificate_id             VARCHAR     NOT NULL,
  field                      VARCHAR     NOT NULL,
  data                       VARCHAR     NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS certificate_data_certificate_id_index ON certificate_data (certificate_id);
CREATE INDEX IF NOT EXISTS certificate_data_block_index ON certificate_data (end_block_num);


CREATE TABLE IF NOT EXISTS standards (
  id                         BIGSERIAL  PRIMARY KEY,
  standard_id                VARCHAR    NOT NULL,
  organization_id            VARCHAR    NOT NULL,
  name                       VARCHAR    NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS standards_id_index ON standards (standard_id);
CREATE INDEX IF NOT EXISTS standards_block_index ON standards (end_block_num);

CREATE TABLE IF NOT EXISTS standard_versions (
  id                         BIGSERIAL   PRIMARY KEY,
  standard_id                VARCHAR     NOT NULL,
  version                    VARCHAR     NOT NULL,
  link                       VARCHAR     NOT NULL,
  description                VARCHAR     NOT NULL,
  approval_date              BIGINT      NOT NULL
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS standard_versions_id_index ON standard_versions (standard_id);
CREATE INDEX IF NOT EXISTS standard_versions_block_index ON standard_versions (end_block_num);

CREATE TABLE IF NOT EXISTS users (
  public_key                 VARCHAR     PRIMARY KEY,
  encrypted_private_key      VARCHAR     NOT NULL,
  username                   VARCHAR     NOT NULL  UNIQUE,
  hashed_password            VARCHAR     NOT NULL
);

CREATE TABLE IF NOT EXISTS requests (
  id                          BIGSERIAL      PRIMARY KEY,
  request_id                  VARCHAR        NOT NULL,
  factory_id                  VARCHAR        NOT NULL,
  standard_id                 VARCHAR        NOT NULL,
  status                      RequestStatus  NOT NULL,
  request_date                BIGINT         NOT NULL
) INHERITS (chain_record);

CREATE TABLE IF NOT EXISTS assertions (
  id                          BIGSERIAL      PRIMARY KEY,
  address                     VARCHAR        NOT NULL,
  assertion_id                VARCHAR        NOT NULL,
  assertor_pub_key            VARCHAR        NOT NULL,
  assertion_type              AssertionType  NOT NULL,
  object_id                   VARCHAR        NOT NULL,
  data_id                     VARCHAR
) INHERITS (chain_record);

CREATE INDEX IF NOT EXISTS assertions_id_index ON assertions (assertion_id);
CREATE INDEX IF NOT EXISTS assertions_address_index ON assertions (address);
CREATE INDEX IF NOT EXISTS assertions_object_id_index ON assertions (object_id);
CREATE INDEX IF NOT EXISTS assertions_block_index ON assertions (end_block_num);