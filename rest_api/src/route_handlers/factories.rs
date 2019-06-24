use std::collections::HashMap;

use database::DbConn;
use database_manager::custom_types::OrganizationTypeEnum;
use database_manager::models::{
    Address, Authorization, Certificate, Contact, Organization, Standard,
};
use database_manager::tables_schema::{
    addresses, authorizations, certificates, contacts, organizations, standards,
};
use diesel::prelude::*;
use errors::ApiError;
use paging::*;
use rocket::request::Form;
use rocket::http::uri::Uri;
use rocket_contrib::json::JsonValue;
use route_handlers::organizations::ApiFactory;

#[derive(Default, FromForm, Clone)]
pub struct FactoryParams {
    name: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    head: Option<i64>,
    expand: Option<bool>,
}

#[get("/factories/<organization_id>")]
pub fn fetch_factory(organization_id: String, conn: DbConn) -> Result<JsonValue, ApiError> {
    fetch_factory_with_head_param(organization_id, None, conn)
}

#[get("/factories/<organization_id>?<params..>")]
pub fn fetch_factory_with_head_param(
    organization_id: String,
    params: Option<Form<FactoryParams>>,
    conn: DbConn,
) -> Result<JsonValue, ApiError> {
    let params = match params {
        Some(param) => param.into_inner(),
        None => Default::default()
    };
    let head_block_num: i64 = get_head_block_num(params.head, &conn)?;

    let factory = organizations::table
        .filter(organizations::organization_type.eq(OrganizationTypeEnum::Factory))
        .filter(organizations::organization_id.eq(organization_id.to_string()))
        .filter(organizations::start_block_num.le(head_block_num))
        .filter(organizations::end_block_num.gt(head_block_num))
        .first::<Organization>(&*conn)
        .optional()
        .map_err(|err| ApiError::InternalError(err.to_string()))?;

    let link = format!("/api/factories/{}?head={}", organization_id, head_block_num);

    match factory {
        Some(factory) => {
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

            let address_results = addresses::table
                .filter(addresses::organization_id.eq(organization_id.to_string()))
                .filter(addresses::start_block_num.le(head_block_num))
                .filter(addresses::end_block_num.gt(head_block_num))
                .first::<Address>(&*conn)
                .optional()
                .map_err(|err| ApiError::InternalError(err.to_string()))?
                .unwrap_or_else(Address::default);

            Ok(json!({
                "data": match params.expand {
                    Some(_) => {
                        let certificate_results = query_certifications(conn, head_block_num, &[organization_id.to_string()])?;

                        ApiFactory::with_certificate_expanded(
                            factory,
                            address_results,
                            contact_results,
                            authorization_results,
                            certificate_results,
                        )
                    }
                    _ => {
                        ApiFactory::from(
                            factory,
                            address_results,
                            contact_results,
                            authorization_results,
                        )
                    }
                },
                "link": link,
                "head": head_block_num,
            }))
        }
        None => Err(ApiError::NotFound(format!(
            "No factory with the organization ID {} exists",
            organization_id
        ))),
    }
}

#[get("/factories")]
pub fn list_factories(conn: DbConn) -> Result<JsonValue, ApiError> {
    query_factories(None, conn)
}

#[get("/factories?<params..>")]
pub fn list_factories_params(params: Option<Form<FactoryParams>>, conn: DbConn) -> Result<JsonValue, ApiError> {
    query_factories(params, conn)
}

fn query_factories(params: Option<Form<FactoryParams>>, conn: DbConn) -> Result<JsonValue, ApiError> {
    let params = match params {
        Some(param) => param.into_inner(),
        None => Default::default()
    };
    let head_block_num: i64 = get_head_block_num(params.head, &conn)?;

    let mut factories_query = organizations::table
        .filter(organizations::start_block_num.le(head_block_num))
        .filter(organizations::end_block_num.gt(head_block_num))
        .filter(organizations::organization_type.eq(OrganizationTypeEnum::Factory))
        .order_by(organizations::organization_id.asc())
        .into_boxed();

    let mut count_query = organizations::table
        .filter(organizations::start_block_num.le(head_block_num))
        .filter(organizations::end_block_num.gt(head_block_num))
        .into_boxed();
    let link_params = params.clone();

    let expand = params.expand.unwrap_or(false);

    if let Some(name) = params.name {
        factories_query = factories_query.filter(organizations::name.eq(name.to_string()));
        count_query = count_query.filter(organizations::name.eq(name.to_string()));
    }

    let total_count = count_query
        .count()
        .get_result(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?;
    let paging_info = apply_paging(link_params, head_block_num, total_count)?;

    factories_query = factories_query.limit(params.limit.unwrap_or(DEFAULT_LIMIT));
    factories_query = factories_query.offset(params.offset.unwrap_or(DEFAULT_OFFSET));

    let factory_results: Vec<Organization> = factories_query.load::<Organization>(&*conn)?;

    let mut contact_results: HashMap<String, Vec<Contact>> = contacts::table
        .filter(contacts::start_block_num.le(head_block_num))
        .filter(contacts::end_block_num.gt(head_block_num))
        .filter(
            contacts::organization_id.eq_any(
                factory_results
                    .iter()
                    .map(|factory| factory.organization_id.to_string())
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
                factory_results
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

    let factory_ids: Vec<String> = factory_results
        .iter()
        .map(|org| org.organization_id.to_string())
        .collect();
    let mut address_results: HashMap<String, Address> = addresses::table
        .filter(addresses::start_block_num.le(head_block_num))
        .filter(addresses::end_block_num.gt(head_block_num))
        .filter(addresses::organization_id.eq_any(&factory_ids))
        .order_by(addresses::organization_id.asc())
        .load::<Address>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, address| {
            acc.insert(address.organization_id.to_string(), address);
            acc
        });

    let mut cert_results: HashMap<String, Vec<(Certificate, Standard, Organization)>> =
        query_certifications(conn, head_block_num, &factory_ids)?
            .into_iter()
            .fold(
                HashMap::new(),
                |mut acc, cert_info: (Certificate, Standard, Organization)| {
                    acc.entry(cert_info.0.factory_id.to_string())
                        .or_insert_with(|| vec![])
                        .push(cert_info);
                    acc
                },
            );

    Ok(json!({
        "data": factory_results.into_iter()
            .map(|factory| {
                let org_id = factory.organization_id.clone();
                if expand {
                    json!(ApiFactory::with_certificate_expanded(
                        factory,
                        address_results.remove(&org_id).unwrap_or_else(Address::default),
                        contact_results.remove(&org_id).unwrap_or_else(|| vec![]),
                        authorization_results.remove(&org_id).unwrap_or_else(|| vec![]),
                        cert_results.remove(&org_id).unwrap_or_else(|| vec![]),
                    ))
                } else {
                    json!(ApiFactory::from(
                        factory,
                        address_results.remove(&org_id).unwrap_or_else(Address::default),
                        contact_results.remove(&org_id).unwrap_or_else(|| vec![]),
                        authorization_results.remove(&org_id).unwrap_or_else(|| vec![]),
                    ))
                }
            }).collect::<Vec<_>>(),
        "link": paging_info.get("link"),
        "head": head_block_num,
        "paging": paging_info.get("paging")
    }))
}

fn query_certifications(
    conn: DbConn,
    head_block_num: i64,
    factory_ids: &[String],
) -> Result<Vec<(Certificate, Standard, Organization)>, ApiError> {
    certificates::table
        .filter(certificates::start_block_num.le(head_block_num))
        .filter(certificates::end_block_num.gt(head_block_num))
        .filter(certificates::factory_id.eq_any(factory_ids))
        .left_join(
            standards::table.on(standards::standard_id
                .eq(certificates::standard_id)
                .and(standards::start_block_num.le(head_block_num))
                .and(standards::end_block_num.gt(head_block_num))),
        )
        .left_join(
            organizations::table.on(organizations::organization_id
                .eq(certificates::certifying_body_id)
                .and(organizations::start_block_num.le(head_block_num))
                .and(organizations::end_block_num.gt(head_block_num))),
        )
        .load::<(Certificate, Option<Standard>, Option<Organization>)>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .map(|(cert, std_opt, org_opt)| {
            Ok((
                cert,
                std_opt.ok_or_else(|| {
                    ApiError::InternalError(
                        "No Standard was provided, but one must exist".to_string(),
                    )
                })?,
                org_opt.ok_or_else(|| {
                    ApiError::InternalError(
                        "No Certifying Body was provided, but one must exist".to_string(),
                    )
                })?,
            ))
        })
        .collect()
}

fn apply_paging(
    params: FactoryParams,
    head: i64,
    total_count: i64,
) -> Result<JsonValue, ApiError> {
    let mut link = String::from("/api/factories?");

    if let Some(name) = params.name {
        link = format!("{}name={}&", link, Uri::percent_encode(&name));
    }
    link = format!("{}head={}&", link, head);

    if let Some(expand) = params.expand {
        link = format!("{}expand={}&", link, expand);
    }

    get_response_paging_info(
        params.limit,
        params.offset,
        link.to_string().clone(),
        total_count,
    )
}
