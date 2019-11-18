use database::DbConn;
use database_manager::models::{Standard, StandardVersion};
use database_manager::tables_schema::standards;
use diesel::prelude::*;
use errors::ApiError;
use paging::get_head_block_num;
use rocket::request::Form;
use rocket_contrib::json::JsonValue;
use std::collections::HashMap;

#[derive(Default, FromForm, Clone)]
pub struct StandardParams {
    organization_id: Option<String>,
    head: Option<i64>,
}

#[derive(Serialize)]
pub struct ApiStandard {
    standard_id: String,
    organization_id: String,
    name: String,
    versions: Vec<ApiVersion>,
}

#[derive(Serialize)]
pub struct ApiVersion {
    version: String,
    external_link: String,
    description: String,
    approval_date: i64,
}

impl From<(Standard, Vec<StandardVersion>)> for ApiStandard {
    fn from(standard_version: (Standard, Vec<StandardVersion>)) -> Self {
        let (standard, version) = standard_version;
        ApiStandard {
            standard_id: standard.standard_id,
            organization_id: standard.organization_id,
            name: standard.name,
            versions: version
                .iter()
                .map(|version| ApiVersion {
                    version: version.version.clone(),
                    external_link: version.link.clone(),
                    description: version.description.clone(),
                    approval_date: version.approval_date,
                })
                .collect::<Vec<ApiVersion>>(),
        }
    }
}

impl<'a> From<(&'a Standard, &'a Vec<StandardVersion>)> for ApiStandard {
    fn from(standard_version: (&Standard, &Vec<StandardVersion>)) -> Self {
        let (standard, version) = standard_version;
        ApiStandard {
            standard_id: standard.standard_id.clone(),
            organization_id: standard.organization_id.clone(),
            name: standard.name.clone(),
            versions: version
                .iter()
                .map(|version| ApiVersion {
                    version: version.version.clone(),
                    external_link: version.link.clone(),
                    description: version.description.clone(),
                    approval_date: version.approval_date,
                })
                .collect::<Vec<ApiVersion>>(),
        }
    }
}

#[get("/standards")]
pub fn list_standards(conn: DbConn) -> Result<JsonValue, ApiError> {
    list_standards_with_params(None, conn)
}

#[get("/standards?<params..>")]
pub fn list_standards_with_params(
    params: Option<Form<StandardParams>>,
    conn: DbConn,
) -> Result<JsonValue, ApiError> {
    let params = match params {
        Some(param) => param.into_inner(),
        None => Default::default(),
    };
    let head_block_num: i64 = get_head_block_num(params.head, &conn)?;
    let mut standards_query = standards::table
        .filter(standards::start_block_num.le(head_block_num))
        .filter(standards::end_block_num.gt(head_block_num))
        .into_boxed();

    if let Some(organization_id) = params.organization_id {
        standards_query =
            standards_query.filter(standards::organization_id.eq(organization_id.to_string()));
    }

    let standards = standards_query
        .load::<Standard>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?
        .into_iter()
        .fold(Vec::new(), |mut acc, standard| {
            acc.push(
                [
                    ("standard_id", standard.standard_id),
                    ("standard_name", standard.name),
                ]
                .iter()
                .cloned()
                .collect::<HashMap<&str, String>>(),
            );
            acc
        });

    Ok(json!({ "data": standards }))
}
