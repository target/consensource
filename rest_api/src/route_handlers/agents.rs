use database::DbConn;
use database_manager::custom_types::OrganizationTypeEnum;
use database_manager::models::{Agent, Organization};
use database_manager::tables_schema::{agents, organizations};
use diesel::prelude::*;
use errors::ApiError;
use paging::*;
use rocket::request::Form;
use rocket_contrib::json::JsonValue;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct ApiOrganization {
    id: String,
    name: String,
    organization_type: OrganizationTypeEnum,
}

impl<'a> From<&'a Organization> for ApiOrganization {
    fn from(org: &'a Organization) -> Self {
        ApiOrganization {
            id: org.organization_id.clone(),
            name: org.name.clone(),
            organization_type: org.organization_type.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct ApiAgent {
    public_key: String,
    name: String,
    created_on: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    organization: Option<ApiOrganization>,
}

impl ApiAgent {
    fn with_org(agent: &Agent, org: &Option<Organization>) -> Self {
        ApiAgent {
            public_key: agent.public_key.clone(),
            name: agent.name.clone(),
            created_on: agent.timestamp,
            organization: org.as_ref().map(ApiOrganization::from),
        }
    }
}

#[get("/agents/<public_key>")]
pub fn fetch_agent(public_key: String, conn: DbConn) -> Result<JsonValue, ApiError> {
    fetch_agent_with_head_param(public_key, None, conn)
}

#[get("/agents/<public_key>?<head_param..>")]
pub fn fetch_agent_with_head_param(
    public_key: String,
    head_param: Option<Form<AgentParams>>,
    conn: DbConn,
) -> Result<JsonValue, ApiError> {
    let head_param = match head_param {
        Some(param) => param.into_inner(),
        None => Default::default()
    };
    let head_block_num: i64 = get_head_block_num(head_param.head, &conn)?;

    let agent = agents::table
        .filter(agents::public_key.eq(public_key.to_string()))
        .filter(agents::start_block_num.le(head_block_num))
        .filter(agents::end_block_num.gt(head_block_num))
        .first::<Agent>(&*conn)
        .optional()
        .map_err(|err| ApiError::InternalError(err.to_string()))?;

    match agent {
        Some(agent) => {
            let org: Option<Organization> =
                if let Some(organization_id) = agent.organization_id.as_ref() {
                    organizations::table
                        .filter(organizations::organization_id.eq(organization_id))
                        .filter(organizations::start_block_num.le(head_block_num))
                        .filter(organizations::end_block_num.gt(head_block_num))
                        .first::<Organization>(&*conn)
                        .optional()
                        .map_err(|err| ApiError::InternalError(err.to_string()))?
                } else {
                    None
                };

            let link = format!("/api/agents/{}?head={}", public_key, head_block_num);
            Ok(json!({
                "data": ApiAgent::with_org(&agent, &org),
                "link": link,
                "head": head_block_num, }))
        }
        None => Err(ApiError::NotFound(format!(
            "No agent with the public key {} exists",
            public_key
        ))),
    }
}

#[derive(Default, FromForm, Clone)]
pub struct AgentParams {
    limit: Option<i64>,
    offset: Option<i64>,
    head: Option<i64>,
}

#[get("/agents")]
pub fn list_agents(conn: DbConn) -> Result<JsonValue, ApiError> {
    list_agents_with_params(None, conn)
}

#[get("/agents?<params..>")]
pub fn list_agents_with_params(params: Option<Form<AgentParams>>, conn: DbConn) -> Result<JsonValue, ApiError> {
    let params = match params {
        Some(param) => param.into_inner(),
        None => Default::default()
    };
    let head_block_num: i64 = get_head_block_num(params.head, &conn)?;

    let mut agents_query = agents::table
        .filter(agents::start_block_num.le(head_block_num))
        .filter(agents::end_block_num.gt(head_block_num))
        .into_boxed();

    let total_count = agents::table
        .filter(agents::start_block_num.le(head_block_num))
        .filter(agents::end_block_num.gt(head_block_num))
        .count()
        .get_result(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?;

    let link_params = params.clone();
    let paging_info = apply_paging(link_params, head_block_num, total_count)?;

    agents_query = agents_query.limit(params.limit.unwrap_or(DEFAULT_LIMIT));
    agents_query = agents_query.offset(params.offset.unwrap_or(DEFAULT_OFFSET));

    let agent_results = agents_query
        .load::<Agent>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?;

    let org_ids: Vec<String> = agent_results
        .iter()
        .map(|agent| agent.organization_id.clone())
        .filter_map(|x| x)
        .collect();

    let mut organization_results: HashMap<String, Organization> = organizations::table
        .filter(organizations::start_block_num.le(head_block_num))
        .filter(organizations::end_block_num.gt(head_block_num))
        .filter(organizations::organization_id.eq_any(org_ids))
        .load::<Organization>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(HashMap::new(), |mut acc, org| {
            acc.insert(org.organization_id.clone(), org);
            acc
        });

    Ok(json!({ "data": agent_results.iter().map(|agent| {
        let org: Option<Organization> = agent.organization_id.as_ref()
            .and_then(|id| organization_results.remove(id));
        ApiAgent::with_org(agent, &org)
    }).collect::<Vec<_>>(),
                    "link": paging_info.get("link"),
                    "head": head_block_num,
                    "paging": paging_info.get("paging") }))
}

fn apply_paging(params: AgentParams, head: i64, total_count: i64) -> Result<JsonValue, ApiError> {
    let link = format!("/api/agents?head={}&", head);

    get_response_paging_info(
        params.limit,
        params.offset,
        link.to_string().clone(),
        total_count,
    )
}
