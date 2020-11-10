-- clear all tables
delete from accreditations;
delete from addresses;
delete from agents;
delete from authorizations;
delete from blocks;
delete from certificate_data;
delete from certificates;
delete from chain_record;
delete from contacts;
delete from organizations;
delete from requests;
delete from standard_versions;
delete from standards;
delete from users;

-- drop roles
drop type Role cascade;
drop type OrganizationType;
drop type RequestStatus;
drop type AssertionType;

-- drop all tables
drop table if exists accreditations;
drop table if exists addresses;
drop table if exists agents;
drop table if exists authorizations;
drop table if exists blocks;
drop table if exists certificate_data;
drop table if exists certificates;
drop table if exists chain_record cascade;
drop table if exists contacts;
drop table if exists organizations;
drop table if exists requests;
drop table if exists standard_versions;
drop table if exists standards;
drop table if exists users;