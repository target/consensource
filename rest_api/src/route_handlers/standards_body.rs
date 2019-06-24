use database::DbConn;
use database_manager::models::{Standard, StandardVersion};
use database_manager::tables_schema::{standard_versions, standards};
use diesel::prelude::*;
use errors::ApiError;
use paging::*;
use rocket::request::Form;
use rocket_contrib::json::JsonValue;
use standards::ApiStandard;
use std::collections::HashMap;

#[derive(Default, FromForm, Clone)]
pub struct StandardBodyParams {
    organization_id: String,
    limit: Option<i64>,
    offset: Option<i64>,
    head: Option<i64>,
}

#[get("/standards_body/standards?<params..>")]
pub fn list_standards_belonging_to_org(
    params: Option<Form<StandardBodyParams>>,
    conn: DbConn,
) -> Result<JsonValue, ApiError> {
    let params = match params {
        Some(param) => param.into_inner(),
        None => Default::default()
    };
    let head_block_num: i64 = get_head_block_num(params.head, &conn)?;

    let link_params = params.clone();
    let total_count = standards::table
        .filter(standards::start_block_num.le(head_block_num))
        .filter(standards::end_block_num.gt(head_block_num))
        .filter(standards::organization_id.eq(params.organization_id.clone()))
        .count()
        .get_result(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?;
    let paging_info = apply_paging(link_params, head_block_num, total_count)?;

    let standards_results = standards::table
        .filter(standards::start_block_num.le(head_block_num))
        .filter(standards::end_block_num.gt(head_block_num))
        .filter(standards::organization_id.eq(params.organization_id))
        .limit(params.limit.unwrap_or(DEFAULT_LIMIT))
        .offset(params.offset.unwrap_or(DEFAULT_OFFSET))
        .load::<Standard>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?;

    let mut standard_version: HashMap<String, Vec<StandardVersion>> = standard_versions::table
        .filter(standard_versions::start_block_num.le(head_block_num))
        .filter(standard_versions::end_block_num.gt(head_block_num))
        .filter(
            standard_versions::standard_id.eq_any(
                standards_results
                    .iter()
                    .map(|standard| standard.standard_id.to_string())
                    .collect::<Vec<String>>(),
            ),
        )
        .load::<StandardVersion>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, standard_version| {
            acc.entry(standard_version.standard_id.to_string())
                .or_insert_with(|| vec![])
                .push(standard_version);
            acc
        });

    Ok(json!({ "data": standards_results.into_iter()
                .map(|standard| {
                     let standard_id = standard.standard_id.clone();
                     ApiStandard::from(
                         (standard,
                         standard_version.remove(&standard_id).map(|mut versions| {
                             versions.sort_by(|v1, v2| v1.approval_date.cmp(&v2.approval_date));
                             versions
                         }).unwrap_or_else(|| vec![])))
                }).collect::<Vec<_>>(),
                "link": paging_info.get("link"),
                "paging":paging_info.get("paging")}))
}

fn apply_paging(
    params: StandardBodyParams,
    head: i64,
    total_count: i64,
) -> Result<JsonValue, ApiError> {
    let mut link = String::from("/api/standards_body/standards?");

    link = format!(
        "{}organization_id={}&head{}=",
        link, params.organization_id, head
    );

    get_response_paging_info(
        params.limit,
        params.offset,
        link.to_string().clone(),
        total_count,
    )
}
