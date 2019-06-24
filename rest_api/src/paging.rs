use database::DbConn;
use database_manager::tables_schema::blocks;
use diesel::dsl::max;
use diesel::prelude::*;
use errors::ApiError;
use rocket_contrib::json::JsonValue;

pub const DEFAULT_LIMIT: i64 = 100;
pub const DEFAULT_OFFSET: i64 = 0;

pub fn get_response_paging_info(
    limit: Option<i64>,
    offset: Option<i64>,
    link: String,
    query_count: i64,
) -> Result<JsonValue, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let offset = offset.unwrap_or(DEFAULT_OFFSET);

    let base_link = format!("{}limit={}&", link, limit);

    let current_link = format!("{}offset={}", base_link, offset);

    let first_link = format!("{}offset=0", base_link);

    let previous_offset = if offset - limit >= 0 {
        offset - limit
    } else {
        0
    };
    let previous_link = format!("{}offset={}", base_link, previous_offset);

    let last_offset = ((query_count - 1) / limit) * limit;
    let last_link = format!("{}offset={}", base_link, last_offset);

    let next_offset = if offset + limit > last_offset {
        last_offset
    } else {
        offset + limit
    };

    let next_link = format!("{}offset={}", base_link, next_offset);

    Ok(json!({ "link": current_link,
    "paging": {
        "offset": offset,
        "limit": limit,
        "total": query_count,
        "first": first_link,
        "prev": previous_link,
        "next": next_link,
        "last": last_link,
    }}))
}

pub fn get_head_block_num(head: Option<i64>, conn: &DbConn) -> Result<i64, ApiError> {
    if let Some(head) = head {
        Ok(head)
    } else {
        blocks::table
            .select(max(blocks::block_num))
            .first::<Option<i64>>(&**conn)
            .map_err(|err| ApiError::InternalError(err.to_string()))
            .and_then(|block_num| block_num.ok_or(ApiError::ServiceUnavailable))
    }
}

#[cfg(test)]
mod tests {
    use paging::*;
    use rocket_contrib::json::JsonValue;

    const TEST_LINK: &str = "/api/test?";

    #[test]
    fn test_default_paging_response() {
        // Create paging response from default limit, default offset, a total of 1000
        let test_paging_response =
            get_response_paging_info(None, None, String::from(TEST_LINK), 1000).unwrap();
        let generated_paging_response =
            create_test_paging_response(DEFAULT_OFFSET, DEFAULT_LIMIT, 100, 0, 900).unwrap();
        assert_eq!(
            test_paging_response.get("link"),
            generated_paging_response.get("link")
        );
        assert!(test_paging_response.get("paging").is_some());
        assert_eq!(
            test_paging_response.get("paging"),
            generated_paging_response.get("paging")
        );
    }

    #[test]
    fn test_50offset_paging_response() {
        // Create paging response from default limit, offset of 50, and a total of 1000
        let test_paging_response =
            get_response_paging_info(None, Some(50), String::from(TEST_LINK), 1000).unwrap();
        let generated_paging_response =
            create_test_paging_response(50, DEFAULT_LIMIT, 150, 0, 900).unwrap();
        assert_eq!(
            test_paging_response.get("link"),
            generated_paging_response.get("link")
        );
        assert!(test_paging_response.get("paging").is_some());
        assert_eq!(
            test_paging_response.get("paging"),
            generated_paging_response.get("paging")
        );
    }

    #[test]
    fn test_550offset_paging_response() {
        // Create paging response from default limit, offset value of 150, and a total of 1000
        let test_paging_response =
            get_response_paging_info(None, Some(550), String::from(TEST_LINK), 1000).unwrap();
        let generated_paging_response =
            create_test_paging_response(550, DEFAULT_LIMIT, 650, 450, 900).unwrap();
        assert_eq!(
            test_paging_response.get("link"),
            generated_paging_response.get("link")
        );
        assert!(test_paging_response.get("paging").is_some());
        assert_eq!(
            test_paging_response.get("paging"),
            generated_paging_response.get("paging")
        );
    }

    #[test]
    fn test_950offset_paging_response() {
        // Create paging response from default limit, offset value of 950, and a total of 1000
        let test_paging_response =
            get_response_paging_info(None, Some(950), String::from(TEST_LINK), 1000).unwrap();
        let generated_paging_response =
            create_test_paging_response(950, DEFAULT_LIMIT, 900, 850, 900).unwrap();
        assert_eq!(
            test_paging_response.get("link"),
            generated_paging_response.get("link")
        );
        assert!(test_paging_response.get("paging").is_some());
        assert_eq!(
            test_paging_response.get("paging"),
            generated_paging_response.get("paging")
        );
    }

    #[test]
    fn test_50limit_paging_response() {
        // Create paging response from default limit, offset of 50, and a total of 1000
        let test_paging_response =
            get_response_paging_info(Some(50), None, String::from(TEST_LINK), 1000).unwrap();
        let generated_paging_response =
            create_test_paging_response(DEFAULT_OFFSET, 50, 50, 0, 950).unwrap();
        assert_eq!(
            test_paging_response.get("link"),
            generated_paging_response.get("link")
        );
        assert!(test_paging_response.get("paging").is_some());
        assert_eq!(
            test_paging_response.get("paging"),
            generated_paging_response.get("paging")
        );
    }

    #[test]
    fn test_50limit_150offset_paging_response() {
        // Create paging response from limit of 50, offset of 150, and total of 1000
        let test_paging_response =
            get_response_paging_info(Some(50), Some(150), String::from(TEST_LINK), 1000).unwrap();
        let generated_paging_response =
            create_test_paging_response(150, 50, 200, 100, 950).unwrap();
        assert_eq!(
            test_paging_response.get("link"),
            generated_paging_response.get("link")
        );
        assert!(test_paging_response.get("paging").is_some());
        assert_eq!(
            test_paging_response.get("paging"),
            generated_paging_response.get("paging")
        );
    }

    fn create_test_paging_response(
        offset: i64,
        limit: i64,
        next_offset: i64,
        previous_offset: i64,
        last_offset: i64,
    ) -> Result<JsonValue, ApiError> {
        // Creates a generated paging response from the limit and offset values passed into the function
        let base_link = format!("{}limit={}&", TEST_LINK, limit);
        let current_link = format!("{}offset={}", base_link, offset);
        let first_link = format!("{}offset=0", base_link);
        let next_link = format!("{}offset={}", base_link, next_offset);
        let previous_link = format!("{}offset={}", base_link, previous_offset);
        let last_link = format!("{}offset={}", base_link, last_offset);

        Ok(json!({ "link": current_link,
        "paging": {
            "offset": offset,
            "limit": limit,
            "total": 1000,
            "first": first_link,
            "prev": previous_link,
            "next": next_link,
            "last": last_link,
        }}))
    }
}
