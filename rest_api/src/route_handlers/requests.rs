use database::DbConn;
use database_manager::custom_types::RequestStatusEnum;
use database_manager::models::{
    Address, Authorization, Contact, Organization, Request, Standard, StandardVersion,
};
use database_manager::tables_schema::{
    addresses, authorizations, contacts, organizations, requests, standard_versions, standards,
};
use diesel::prelude::*;
use errors::ApiError;
use paging::*;
use rocket::request::Form;
use rocket::http::uri::Uri;
use rocket_contrib::json::JsonValue;
use std::collections::HashMap;

use route_handlers::organizations::ApiFactory;
use route_handlers::standards::ApiStandard;

#[get("/requests/<request_id>")]
pub fn fetch_request(request_id: String, conn: DbConn) -> Result<JsonValue, ApiError> {
    fetch_request_with_head_param(request_id, None, conn)
}

#[get("/requests/<request_id>?<head_param..>")]
pub fn fetch_request_with_head_param(
    request_id: String,
    head_param: Option<Form<CertRequestParams>>,
    conn: DbConn,
) -> Result<JsonValue, ApiError> {
    let head_param = match head_param {
        Some(param) => param.into_inner(),
        None => Default::default()
    };
    let head_block_num: i64 = get_head_block_num(head_param.head, &conn)?;

    let request = requests::table
        .filter(requests::request_id.eq(request_id.to_string()))
        .filter(requests::start_block_num.le(head_block_num))
        .filter(requests::end_block_num.gt(head_block_num))
        .first::<Request>(&*conn)
        .optional()
        .map_err(|err| ApiError::InternalError(err.to_string()))?;

    let mut link = format!("/api/requests/{}?head={}", request_id, head_block_num);
    if let Some(expand) = head_param.expand {
        link = format!("{}&expand={}", link, expand);
    }

    match request {
        Some(request) => {
            if let Some(true) = head_param.expand {
                let expansions = fetch_expansions(
                    &conn,
                    &[request.factory_id.clone()],
                    &[request.standard_id.clone()],
                    head_block_num,
                )?;
                let factory_results = expansions.factory_results;
                let contact_results = expansions.contact_results;
                let authorization_results = expansions.authorization_results;
                let address_results = expansions.address_results;
                let standard_results = expansions.standard_results;
                let standard_version_results = expansions.standard_version_results;
                let factory_id = request.factory_id.clone();
                let standard_id = request.standard_id.clone();
                let factory = ApiFactory::from_ref(
                    factory_results
                        .get(&factory_id)
                        .expect("Error getting factory"),
                    address_results
                        .get(&factory_id)
                        .unwrap_or(&Address::default()),
                    contact_results.get(&factory_id).unwrap_or(&vec![]),
                    authorization_results.get(&factory_id).unwrap_or(&vec![]),
                );
                let standard = ApiStandard::from((
                    standard_results
                        .get(&standard_id)
                        .expect("Error getting standard"),
                    standard_version_results
                        .get(&standard_id)
                        .expect("Error getting standard versions"),
                ));

                Ok(json!({
                    "data": ApiRequest::with_expansion(request, factory, standard),
                    "link": link,
                    "head": head_block_num,
                }))
            } else {
                Ok(json!({
                    "data": ApiRequest::from(request),
                    "link": link,
                    "head": head_block_num,
                }))
            }
        }
        None => Err(ApiError::NotFound(format!(
            "No certification request with the ID {} exists",
            request_id
        ))),
    }
}

#[derive(Default, FromForm, Clone)]
pub struct CertRequestParams {
    factory_id: Option<String>,
    expand: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
    head: Option<i64>,
}

#[derive(Serialize)]
pub struct ApiRequest {
    id: String,
    factory: FactoryExpansion,
    standard: StandardExpansion,
    status: RequestStatusEnum,
    request_date: i64,
}

impl ApiRequest {
    pub fn from(req: Request) -> Self {
        ApiRequest {
            id: req.request_id,
            factory: FactoryExpansion::Ref {
                id: req.factory_id.clone(),
                link: format!(
                    "/api/organizations/{}",
                    Uri::percent_encode(&req.factory_id)
                ),
            },
            standard: StandardExpansion::Ref {
                id: req.standard_id.clone(),
                link: format!("/api/standards/{}", Uri::percent_encode(&req.standard_id)),
            },
            status: req.status,
            request_date: req.request_date,
        }
    }

    pub fn with_expansion(req: Request, factory: ApiFactory, standard: ApiStandard) -> Self {
        ApiRequest {
            id: req.request_id,
            factory: FactoryExpansion::Expanded(Box::new(factory)),
            standard: StandardExpansion::Expanded(standard),
            status: req.status,
            request_date: req.request_date,
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum FactoryExpansion {
    Ref { id: String, link: String },
    Expanded(Box<ApiFactory>),
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum StandardExpansion {
    Ref { id: String, link: String },
    Expanded(ApiStandard),
}

#[get("/requests")]
pub fn list_requests(conn: DbConn) -> Result<JsonValue, ApiError> {
    query_requests(None, conn)
}

#[get("/requests?<params..>")]
pub fn list_request_with_params(
    params: Option<Form<CertRequestParams>>,
    conn: DbConn,
) -> Result<JsonValue, ApiError> {

    query_requests(params, conn)
}

fn query_requests(params: Option<Form<CertRequestParams>>, conn: DbConn) -> Result<JsonValue, ApiError> {
    let params = match params {
        Some(param) => param.into_inner(),
        None => Default::default()
    };
    let head_block_num: i64 = get_head_block_num(params.head, &conn)?;
    let expand = params.expand.unwrap_or(false);

    let mut requests_query = requests::table
        .filter(requests::start_block_num.le(head_block_num))
        .filter(requests::end_block_num.gt(head_block_num))
        .filter(requests::status.ne(RequestStatusEnum::Closed))
        .filter(requests::status.ne(RequestStatusEnum::Certified))
        .into_boxed();

    let mut count_query = requests::table
        .filter(requests::start_block_num.le(head_block_num))
        .filter(requests::end_block_num.gt(head_block_num))
        .into_boxed();
    let link_params = params.clone();

    if let Some(factory_id) = params.factory_id {
        requests_query = requests_query.filter(requests::factory_id.eq(factory_id.to_string()));
        count_query = count_query.filter(requests::factory_id.eq(factory_id.to_string()));
    }

    let total_count = count_query
        .count()
        .get_result(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?;
    let paging_info = apply_paging(link_params, head_block_num, total_count)?;

    requests_query = requests_query.limit(params.limit.unwrap_or(DEFAULT_LIMIT));
    requests_query = requests_query.offset(params.offset.unwrap_or(DEFAULT_OFFSET));

    let request_results = requests_query
        .load::<Request>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?;

    if expand {
        let factory_ids = request_results
            .iter()
            .map(|request| request.factory_id.to_string())
            .collect::<Vec<String>>();
        let standard_ids = request_results
            .iter()
            .map(|request| request.standard_id.to_string())
            .collect::<Vec<String>>();
        let expansions = fetch_expansions(&conn, &factory_ids, &standard_ids, head_block_num)?;
        let factory_results = expansions.factory_results;
        let contact_results = expansions.contact_results;
        let authorization_results = expansions.authorization_results;
        let address_results = expansions.address_results;
        let standard_results = expansions.standard_results;
        let standard_version_results = expansions.standard_version_results;

        Ok(json!({
            "data": request_results.into_iter()
                .map(|request| {
                    let factory_id = request.factory_id.clone();
                    let standard_id = request.standard_id.clone();
                    let factory = ApiFactory::from_ref(
                        factory_results.get(&factory_id).expect("Error getting factory"),
                        address_results.get(&factory_id).unwrap_or(&Address::default()),
                        contact_results.get(&factory_id).unwrap_or(&vec![]),
                        authorization_results.get(&factory_id).unwrap_or(&vec![]));
                    let standard = ApiStandard::from((
                        standard_results.get(&standard_id).expect("Error getting standard"),
                        standard_version_results.get(&standard_id).expect("Error getting standard versions")
                    ));
                    json!(ApiRequest::with_expansion(request, factory, standard))
                }).collect::<Vec<_>>(),
            "link": paging_info.get("link"),
            "head": head_block_num,
            "paging":paging_info.get("paging")
        }))
    } else {
        Ok(json!({
            "data": request_results.into_iter()
                .map(ApiRequest::from).collect::<Vec<_>>(),
            "link": paging_info.get("link"),
            "head": head_block_num,
            "paging":paging_info.get("paging")
        }))
    }
}

pub struct Expansions {
    factory_results: HashMap<String, Organization>,
    contact_results: HashMap<String, Vec<Contact>>,
    authorization_results: HashMap<String, Vec<Authorization>>,
    address_results: HashMap<String, Address>,
    standard_results: HashMap<String, Standard>,
    standard_version_results: HashMap<String, Vec<StandardVersion>>,
}

fn fetch_expansions(
    conn: &DbConn,
    factory_ids: &[String],
    standard_ids: &[String],
    head_block_num: i64,
) -> Result<
    Expansions,
    ApiError,
> {
    let factory_results: HashMap<String, Organization> = organizations::table
        .filter(organizations::start_block_num.le(head_block_num))
        .filter(organizations::end_block_num.gt(head_block_num))
        .filter(organizations::organization_id.eq_any(factory_ids))
        .order_by(organizations::organization_id.asc())
        .load::<Organization>(&**conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, factory| {
            acc.insert(factory.organization_id.to_string(), factory);
            acc
        });

    let contact_results: HashMap<String, Vec<Contact>> = contacts::table
        .filter(contacts::start_block_num.le(head_block_num))
        .filter(contacts::end_block_num.gt(head_block_num))
        .filter(contacts::organization_id.eq_any(factory_ids))
        .order_by(contacts::organization_id.asc())
        .load::<Contact>(&**conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, contact| {
            acc.entry(contact.organization_id.to_string())
                .or_insert_with(|| vec![])
                .push(contact);
            acc
        });

    let authorization_results: HashMap<String, Vec<Authorization>> = authorizations::table
        .filter(authorizations::start_block_num.le(head_block_num))
        .filter(authorizations::end_block_num.gt(head_block_num))
        .filter(authorizations::organization_id.eq_any(factory_ids))
        .order_by(authorizations::organization_id.asc())
        .load::<Authorization>(&**conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, authorization| {
            acc.entry(authorization.organization_id.to_string())
                .or_insert_with(|| vec![])
                .push(authorization);
            acc
        });

    let address_results: HashMap<String, Address> = addresses::table
        .filter(addresses::start_block_num.le(head_block_num))
        .filter(addresses::end_block_num.gt(head_block_num))
        .filter(addresses::organization_id.eq_any(factory_ids))
        .order_by(addresses::organization_id.asc())
        .load::<Address>(&**conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, address| {
            acc.insert(address.organization_id.to_string(), address);
            acc
        });

    let standard_results: HashMap<String, Standard> = standards::table
        .filter(standards::start_block_num.le(head_block_num))
        .filter(standards::end_block_num.gt(head_block_num))
        .filter(standards::standard_id.eq_any(standard_ids))
        .order_by(standards::standard_id.asc())
        .load::<Standard>(&**conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, standard| {
            acc.insert(standard.standard_id.to_string(), standard);
            acc
        });

    let standard_version_results: HashMap<String, Vec<StandardVersion>> = standard_versions::table
        .filter(standard_versions::standard_id.eq_any(standard_ids))
        .order_by(standard_versions::approval_date.desc())
        .load::<StandardVersion>(&**conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, standard_version| {
            acc.entry(standard_version.standard_id.to_string())
                .or_insert_with(|| vec![])
                .push(standard_version);
            acc
        });
    let expansions = Expansions {
        factory_results,
        contact_results,
        authorization_results,
        address_results,
        standard_results,
        standard_version_results,
    };

    Ok(expansions)
}

fn apply_paging(
    params: CertRequestParams,
    head: i64,
    total_count: i64,
) -> Result<JsonValue, ApiError> {
    let mut link = String::from("/api/requests?");

    if let Some(factory_id) = params.factory_id {
        link = format!("{}factory_id={}&", link, Uri::percent_encode(&factory_id));
    }
    if let Some(expand) = params.expand {
        link = format!("{}expand={}&", link, expand);
    }
    link = format!("{}head={}&", link, head);

    get_response_paging_info(
        params.limit,
        params.offset,
        link.to_string().clone(),
        total_count,
    )
}
