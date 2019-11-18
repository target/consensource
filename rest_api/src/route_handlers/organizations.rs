use database::DbConn;
use database_manager::custom_types::OrganizationTypeEnum;
use database_manager::custom_types::RoleEnum;
use database_manager::models::{
    Address, Authorization, Certificate, Contact, Organization, Standard,
};
use database_manager::tables_schema::{addresses, authorizations, contacts, organizations};
use diesel::prelude::*;
use errors::ApiError;
use paging::*;
use rocket::http::uri::Uri;
use rocket::request::Form;
use rocket_contrib::json::JsonValue;
use route_handlers::certificates::ApiCertificate;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct ApiAddress {
    street_line_1: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    street_line_2: Option<String>,
    city: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_province: Option<String>,
    country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    postal_code: Option<String>,
}

impl ApiAddress {
    fn from(db_address: Address) -> Self {
        ApiAddress {
            street_line_1: db_address.street_line_1,
            street_line_2: db_address.street_line_2,
            city: db_address.city,
            state_province: db_address.state_province,
            country: db_address.country,
            postal_code: db_address.postal_code,
        }
    }

    fn from_ref(db_address: &Address) -> Self {
        ApiAddress {
            street_line_1: db_address.street_line_1.clone(),
            street_line_2: db_address.street_line_2.clone(),
            city: db_address.city.clone(),
            state_province: db_address.state_province.clone(),
            country: db_address.country.clone(),
            postal_code: db_address.postal_code.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct ApiAuthorization {
    public_key: String,
    role: RoleEnum,
}

impl ApiAuthorization {
    fn from(db_authorization: Authorization) -> Self {
        ApiAuthorization {
            public_key: db_authorization.public_key,
            role: db_authorization.role,
        }
    }

    fn from_ref(db_authorization: &Authorization) -> Self {
        ApiAuthorization {
            public_key: db_authorization.public_key.clone(),
            role: db_authorization.role.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct ApiContact {
    name: String,
    language_code: String,
    phone_number: String,
}

impl ApiContact {
    fn from(db_contact: Contact) -> Self {
        ApiContact {
            name: db_contact.name,
            language_code: db_contact.language_code,
            phone_number: db_contact.phone_number,
        }
    }

    fn from_ref(db_contact: &Contact) -> Self {
        ApiContact {
            name: db_contact.name.clone(),
            language_code: db_contact.language_code.clone(),
            phone_number: db_contact.phone_number.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct ApiFactory {
    id: String,
    name: String,
    contacts: Vec<ApiContact>,
    authorizations: Vec<ApiAuthorization>,
    address: ApiAddress,
    #[serde(skip_serializing_if = "Option::is_none")]
    certificates: Option<Vec<ApiCertificate>>,
    organization_type: OrganizationTypeEnum,
}

impl ApiFactory {
    pub fn from(
        db_organization: Organization,
        db_address: Address,
        db_contacts: Vec<Contact>,
        db_authorizations: Vec<Authorization>,
    ) -> Self {
        ApiFactory {
            id: db_organization.organization_id.to_string(),
            name: db_organization.name,
            contacts: db_contacts.into_iter().map(ApiContact::from).collect(),
            authorizations: db_authorizations
                .into_iter()
                .map(ApiAuthorization::from)
                .collect(),
            address: ApiAddress::from(db_address),
            certificates: None,
            organization_type: db_organization.organization_type,
        }
    }

    pub fn with_certificate_expanded(
        db_organization: Organization,
        db_address: Address,
        db_contacts: Vec<Contact>,
        db_authorizations: Vec<Authorization>,
        db_certificates: Vec<(Certificate, Standard, Organization)>,
    ) -> Self {
        let factory = db_organization.clone();
        ApiFactory {
            id: db_organization.organization_id.to_string(),
            name: db_organization.name,
            contacts: db_contacts.into_iter().map(ApiContact::from).collect(),
            authorizations: db_authorizations
                .into_iter()
                .map(ApiAuthorization::from)
                .collect(),
            address: ApiAddress::from(db_address),
            organization_type: db_organization.organization_type,
            certificates: Some(
                db_certificates
                    .into_iter()
                    .map(|(cert, standard, auditor)| {
                        ApiCertificate::from((cert, factory.clone(), standard, auditor))
                    })
                    .collect(),
            ),
        }
    }

    pub fn from_ref(
        db_organization: &Organization,
        db_address: &Address,
        db_contacts: &[Contact],
        db_authorizations: &[Authorization],
    ) -> Self {
        ApiFactory {
            id: db_organization.organization_id.to_string(),
            name: db_organization.name.to_string(),
            contacts: db_contacts
                .iter()
                .map(|contact| ApiContact::from_ref(contact))
                .collect(),
            authorizations: db_authorizations
                .iter()
                .map(|auth| ApiAuthorization::from_ref(auth))
                .collect(),
            address: ApiAddress::from_ref(db_address),
            certificates: None,
            organization_type: db_organization.organization_type.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct ApiCertifyingBody {
    id: String,
    name: String,
    contacts: Vec<ApiContact>,
    authorizations: Vec<ApiAuthorization>,
    organization_type: OrganizationTypeEnum,
}

impl ApiCertifyingBody {
    fn from(
        db_organization: Organization,
        db_contacts: Vec<Contact>,
        db_authorizations: Vec<Authorization>,
    ) -> Self {
        ApiCertifyingBody {
            id: db_organization.organization_id.to_string(),
            name: db_organization.name,
            contacts: db_contacts.into_iter().map(ApiContact::from).collect(),
            authorizations: db_authorizations
                .into_iter()
                .map(ApiAuthorization::from)
                .collect(),
            organization_type: db_organization.organization_type,
        }
    }
}

#[derive(Serialize)]
pub struct ApiStandardsBody {
    id: String,
    name: String,
    contacts: Vec<ApiContact>,
    authorizations: Vec<ApiAuthorization>,
    organization_type: OrganizationTypeEnum,
}

impl ApiStandardsBody {
    fn from(
        db_organization: Organization,
        db_contacts: Vec<Contact>,
        db_authorizations: Vec<Authorization>,
    ) -> Self {
        ApiStandardsBody {
            id: db_organization.organization_id.to_string(),
            name: db_organization.name,
            contacts: db_contacts.into_iter().map(ApiContact::from).collect(),
            authorizations: db_authorizations
                .into_iter()
                .map(ApiAuthorization::from)
                .collect(),
            organization_type: db_organization.organization_type,
        }
    }
}

#[get("/organizations/<organization_id>")]
pub fn fetch_organization(organization_id: String, conn: DbConn) -> Result<JsonValue, ApiError> {
    fetch_organization_with_params(organization_id, None, conn)
}

#[get("/organizations/<organization_id>?<head_param..>")]
pub fn fetch_organization_with_params(
    organization_id: String,
    head_param: Option<Form<OrganizationParams>>,
    conn: DbConn,
) -> Result<JsonValue, ApiError> {
    let head_param = match head_param {
        Some(param) => param.into_inner(),
        None => Default::default(),
    };
    let head_block_num: i64 = get_head_block_num(head_param.head, &conn)?;
    let link = format!(
        "/api/organizations/{}?head={}",
        organization_id, head_block_num
    );

    let org = organizations::table
        .filter(organizations::organization_id.eq(organization_id.to_string()))
        .filter(organizations::start_block_num.le(head_block_num))
        .filter(organizations::end_block_num.gt(head_block_num))
        .first::<Organization>(&*conn)
        .optional()
        .map_err(|err| ApiError::InternalError(err.to_string()))?;

    match org {
        Some(org) => {
            let contact_results: Vec<Contact> = contacts::table
                .filter(contacts::organization_id.eq(organization_id.to_string()))
                .filter(contacts::start_block_num.le(head_block_num))
                .filter(contacts::end_block_num.gt(head_block_num))
                .load::<Contact>(&*conn)
                .map_err(|err| ApiError::InternalError(err.to_string()))?;

            let authorization_results: Vec<Authorization> = authorizations::table
                .filter(authorizations::organization_id.eq(organization_id.to_string()))
                .filter(authorizations::start_block_num.le(head_block_num))
                .filter(authorizations::end_block_num.gt(head_block_num))
                .load::<Authorization>(&*conn)
                .map_err(|err| ApiError::InternalError(err.to_string()))?;

            let data = match org.organization_type {
                OrganizationTypeEnum::Factory => {
                    let address_results = addresses::table
                        .filter(addresses::organization_id.eq(organization_id.to_string()))
                        .filter(addresses::start_block_num.le(head_block_num))
                        .filter(addresses::end_block_num.gt(head_block_num))
                        .first::<Address>(&*conn)
                        .optional()
                        .map_err(|err| ApiError::InternalError(err.to_string()))?
                        .unwrap_or_else(Address::default);
                    json!(ApiFactory::from(
                        org,
                        address_results,
                        contact_results,
                        authorization_results
                    ))
                }
                OrganizationTypeEnum::CertifyingBody => json!(ApiCertifyingBody::from(
                    org,
                    contact_results,
                    authorization_results
                )),
                OrganizationTypeEnum::StandardsBody => json!(ApiStandardsBody::from(
                    org,
                    contact_results,
                    authorization_results
                )),
                OrganizationTypeEnum::UnsetType => json!({}),
            };

            Ok(json!({ "data": data,
                            "link": link,
                            "head": head_block_num,}))
        }
        None => Err(ApiError::NotFound(format!(
            "No organization with the organization ID {} exists",
            organization_id
        ))),
    }
}

#[derive(Default, FromForm, Clone)]
pub struct OrganizationParams {
    name: Option<String>,
    organization_type: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
    head: Option<i64>,
}

#[get("/organizations")]
pub fn list_organizations(conn: DbConn) -> Result<JsonValue, ApiError> {
    list_organizations_with_params(None, conn)
}

#[get("/organizations?<params..>")]
pub fn list_organizations_with_params(
    params: Option<Form<OrganizationParams>>,
    conn: DbConn,
) -> Result<JsonValue, ApiError> {
    let params = match params {
        Some(param) => param.into_inner(),
        None => Default::default(),
    };
    let head_block_num: i64 = get_head_block_num(params.head, &conn)?;

    let mut organizations_query = organizations::table
        .filter(organizations::start_block_num.le(head_block_num))
        .filter(organizations::end_block_num.gt(head_block_num))
        .order_by(organizations::organization_id.asc())
        .into_boxed();

    let mut count_query = organizations::table
        .filter(organizations::start_block_num.le(head_block_num))
        .filter(organizations::end_block_num.gt(head_block_num))
        .into_boxed();
    let link_params = params.clone();

    if let Some(name) = params.name {
        organizations_query = organizations_query.filter(organizations::name.eq(name.to_string()));
        count_query = count_query.filter(organizations::name.eq(name.to_string()));
    }
    if let Some(organization_type) = params.organization_type {
        let org_type = match organization_type {
            1 => OrganizationTypeEnum::CertifyingBody,
            _ => OrganizationTypeEnum::StandardsBody,
        };

        organizations_query =
            organizations_query.filter(organizations::organization_type.eq(org_type.clone()));
        count_query = count_query.filter(organizations::organization_type.eq(org_type.clone()));
    }

    let total_count = count_query
        .count()
        .get_result(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?;
    let paging_info = apply_paging(link_params, head_block_num, total_count)?;

    organizations_query = organizations_query.limit(params.limit.unwrap_or(DEFAULT_LIMIT));
    organizations_query = organizations_query.offset(params.offset.unwrap_or(DEFAULT_OFFSET));

    let organization_results: Vec<Organization> =
        organizations_query.load::<Organization>(&*conn)?;

    let mut contact_results: HashMap<String, Vec<Contact>> = contacts::table
        .filter(contacts::start_block_num.le(head_block_num))
        .filter(contacts::end_block_num.gt(head_block_num))
        .filter(
            contacts::organization_id.eq_any(
                organization_results
                    .iter()
                    .map(|org| org.organization_id.to_string())
                    .collect::<Vec<String>>(),
            ),
        )
        .order_by(contacts::organization_id.asc())
        .load::<Contact>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, contact| {
            acc.entry(contact.organization_id.to_string())
                .or_insert_with(|| vec![])
                .push(contact);
            acc
        });

    let mut authorization_results: HashMap<String, Vec<Authorization>> = authorizations::table
        .filter(authorizations::start_block_num.le(head_block_num))
        .filter(authorizations::end_block_num.gt(head_block_num))
        .filter(
            authorizations::organization_id.eq_any(
                organization_results
                    .iter()
                    .map(|org| org.organization_id.to_string())
                    .collect::<Vec<String>>(),
            ),
        )
        .order_by(authorizations::organization_id.asc())
        .load::<Authorization>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, authorization| {
            acc.entry(authorization.organization_id.to_string())
                .or_insert_with(|| vec![])
                .push(authorization);
            acc
        });

    let mut address_results: HashMap<String, Address> = addresses::table
        .filter(addresses::start_block_num.le(head_block_num))
        .filter(addresses::end_block_num.gt(head_block_num))
        .filter(
            addresses::organization_id.eq_any(
                organization_results
                    .iter()
                    .map(|org| org.organization_id.to_string())
                    .collect::<Vec<String>>(),
            ),
        )
        .order_by(addresses::organization_id.asc())
        .load::<Address>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, address| {
            acc.insert(address.organization_id.to_string(), address);
            acc
        });

    Ok(json!({
        "data": organization_results.into_iter()
            .map(|org| {
                let org_id = org.organization_id.clone();
                match org.organization_type {
                    OrganizationTypeEnum::Factory => {
                        json!(ApiFactory::from(
                            org,
                            address_results.remove(&org_id).unwrap_or_else(Address::default),
                            contact_results.remove(&org_id).unwrap_or_else(|| vec![]),
                            authorization_results.remove(&org_id).unwrap_or_else(|| vec![]),
                        ))
                    }
                    OrganizationTypeEnum::CertifyingBody => {
                        json!(ApiCertifyingBody::from(
                            org,
                            contact_results.remove(&org_id).unwrap_or_else(|| vec![]),
                            authorization_results.remove(&org_id).unwrap_or_else(|| vec![]),
                        ))
                    }
                    OrganizationTypeEnum::StandardsBody => {
                        json!(ApiStandardsBody::from(
                            org,
                            contact_results.remove(&org_id).unwrap_or_else(|| vec![]),
                            authorization_results.remove(&org_id).unwrap_or_else(|| vec![]),
                        ))
                    }
                    OrganizationTypeEnum::UnsetType => json!({})
                }
            }).collect::<Vec<_>>(),
        "link": paging_info.get("link"),
        "head": head_block_num,
        "paging": paging_info.get("paging")
    }))
}

fn apply_paging(
    params: OrganizationParams,
    head: i64,
    total_count: i64,
) -> Result<JsonValue, ApiError> {
    let mut link = String::from("/api/organizations?");

    if let Some(name) = params.name {
        link = format!("{}name={}&", link, Uri::percent_encode(&name));
    }
    link = format!("{}head={}&", link, head);

    get_response_paging_info(
        params.limit,
        params.offset,
        link.to_string().clone(),
        total_count,
    )
}
