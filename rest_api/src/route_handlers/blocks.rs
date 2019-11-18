use database::{DbConn, PgPool};
use database_manager::models::Block;
use database_manager::tables_schema::blocks;
use diesel::prelude::*;
use errors::ApiError;
use hyper_sse::Server;
use paging::*;
use rocket::request::Form;
use rocket_contrib::json::JsonValue;
use std::{thread, time};

const DEFAULT_CHANNEL: u8 = 0;

lazy_static! {
    static ref PUSH_SERVER: Server<u8> = Server::new();
}

pub struct BlockWatcher {
    db_pool: PgPool,
    block_queue: Vec<Block>,
    last_block_height: i64,
}

impl Clone for BlockWatcher {
    fn clone(&self) -> Self {
        BlockWatcher {
            db_pool: self.db_pool.clone(),
            block_queue: vec![],
            last_block_height: -1,
        }
    }
}

impl BlockWatcher {
    /// Constructs a new BlockWatcher
    pub fn new(db_pool: PgPool) -> Self {
        BlockWatcher {
            db_pool,
            block_queue: vec![],
            last_block_height: -1,
        }
    }

    /// Returns the next block, if there is one.
    pub fn take(&mut self) -> Option<Block> {
        if self.block_queue.is_empty() {
            if let Err(err) = self.load_block_queue() {
                error!("Unable to load blocks: {:?}", err);
            }
        }
        self.block_queue.pop()
    }

    fn load_block_queue(&mut self) -> Result<(), WatchError> {
        let db_conn = self
            .db_pool
            .get()
            .map_err(|err| WatchError::ConnectionError(format!("{:?}", err)))?;
        if self.last_block_height < 0 {
            let block: Option<Block> = blocks::table
                .order(blocks::block_num.desc())
                .first(&*db_conn)
                .optional()?;

            if let Some(block) = block {
                self.block_queue.push(block);
            }
        } else {
            let mut blocks: Vec<Block> = blocks::table
                .filter(blocks::block_num.gt(self.last_block_height))
                .order(blocks::block_num.asc())
                .load(&*db_conn)?;

            if !blocks.is_empty() {
                self.block_queue.append(&mut blocks);
            }
        }

        if let Some(block) = self.block_queue.last() {
            self.last_block_height = block.block_num;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum WatchError {
    ConnectionError(String),
    QueryError(String),
}

impl From<::diesel::result::Error> for WatchError {
    fn from(err: ::diesel::result::Error) -> Self {
        WatchError::QueryError(format!("{:?}", err))
    }
}

pub struct WatcherThread {
    join_handle: thread::JoinHandle<()>,
}

impl WatcherThread {
    pub fn run(block_watcher: BlockWatcher, interval: u64, host: &str, port: u16) -> Self {
        let interval = time::Duration::from_millis(interval);
        let mut watcher = block_watcher.clone();
        thread::spawn(move || loop {
            if let Some(block) = watcher.take() {
                debug!("Sending {:?}", block);
                if let Err(err) = PUSH_SERVER.push(DEFAULT_CHANNEL, "block-event", &block) {
                    warn!("Unable to push block-event: {:?}", err);
                };
            } else {
                thread::sleep(interval);
            }
        });

        error!("Starting SSE server on {}:{}", host, port);
        WatcherThread {
            join_handle: start_sse_server(host, port),
        }
    }

    pub fn join(self) -> thread::Result<()> {
        self.join_handle.join()
    }
}

fn start_sse_server(host: &str, port: u16) -> thread::JoinHandle<()> {
    PUSH_SERVER.spawn(
        format!("{}:{}", host, port)
            .parse()
            .expect("Should have been a valid address"),
    )
}

#[get("/blocks/<block_id>")]
pub fn fetch_block(block_id: String, conn: DbConn) -> Result<JsonValue, ApiError> {
    fetch_block_with_head_param(block_id, None, conn)
}

#[get("/blocks/<block_id>?<head_param..>")]
pub fn fetch_block_with_head_param(
    block_id: String,
    head_param: Option<Form<BlockParams>>,
    conn: DbConn,
) -> Result<JsonValue, ApiError> {
    let head_param = match head_param {
        Some(param) => param.into_inner(),
        None => Default::default(),
    };
    let head_block_num: i64 = get_head_block_num(head_param.head, &conn)?;

    let block = blocks::table
        .filter(blocks::block_id.eq(block_id.to_string()))
        .filter(blocks::block_num.le(head_block_num))
        .first::<Block>(&*conn)
        .optional()
        .map_err(|err| ApiError::InternalError(err.to_string()))?;

    let link = format!("/api/blocks/{}", block_id);

    match block {
        Some(block) => Ok(json!({
                "data": block,
                "link": link,
                "head": head_block_num, })),
        None => Err(ApiError::NotFound(format!(
            "No block with the ID {} exists",
            block_id
        ))),
    }
}

#[derive(Default, FromForm, Clone)]
pub struct BlockParams {
    limit: Option<i64>,
    offset: Option<i64>,
    head: Option<i64>,
}

#[get("/blocks")]
pub fn list_blocks(conn: DbConn) -> Result<JsonValue, ApiError> {
    list_blocks_with_params(None, conn)
}

#[get("/blocks?<params..>")]
pub fn list_blocks_with_params(
    params: Option<Form<BlockParams>>,
    conn: DbConn,
) -> Result<JsonValue, ApiError> {
    let params = match params {
        Some(param) => param.into_inner(),
        None => Default::default(),
    };
    let head_block_num: i64 = get_head_block_num(params.head, &conn)?;

    let mut blocks_query = blocks::table
        .filter(blocks::block_num.le(head_block_num))
        .into_boxed();

    let total_count = blocks::table
        .filter(blocks::block_num.le(head_block_num))
        .into_boxed()
        .count()
        .get_result(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?;
    let link_params = params.clone();
    let paging_info = apply_paging(link_params, head_block_num, total_count)?;

    blocks_query = blocks_query.limit(params.limit.unwrap_or(DEFAULT_LIMIT));
    blocks_query = blocks_query.offset(params.offset.unwrap_or(DEFAULT_OFFSET));

    let blocks = blocks_query
        .load::<Block>(&*conn)
        .map_err(|err| ApiError::InternalError(err.to_string()))?;

    Ok(json!({ "data": blocks,
                    "link": paging_info.get("link"),
                    "head": head_block_num,
                    "paging": paging_info.get("paging") }))
}

fn apply_paging(params: BlockParams, head: i64, total_count: i64) -> Result<JsonValue, ApiError> {
    let link = format!("/api/blocks?head={}&", head);

    get_response_paging_info(
        params.limit,
        params.offset,
        link.to_string().clone(),
        total_count,
    )
}
