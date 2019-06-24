use errors::ApiError;
use protobuf;
use protobuf::ProtobufEnum;
use rocket::Data;
use rocket::State;
use rocket::request::Form;
use rocket_contrib::json::JsonValue;
use sawtooth_sdk::messages::batch::BatchList;
use sawtooth_sdk::messages::client_batch_submit::{
    ClientBatchStatus, ClientBatchStatusRequest, ClientBatchStatusResponse,
    ClientBatchStatusResponse_Status, ClientBatchStatus_InvalidTransaction,
    ClientBatchSubmitRequest, ClientBatchSubmitResponse, ClientBatchSubmitResponse_Status,
};
use sawtooth_sdk::messages::validator::Message_MessageType;
use sawtooth_sdk::messaging::stream::MessageConnection;
use sawtooth_sdk::messaging::stream::MessageSender;
use sawtooth_sdk::messaging::zmq_stream::ZmqMessageConnection;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::io::Read;
use uuid;

struct InvalidTransactionWrapper(ClientBatchStatus_InvalidTransaction);
impl Serialize for InvalidTransactionWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("InvalidTransactionWrapper", 3)?;
        state.serialize_field("id", &self.0.get_transaction_id())?;
        state.serialize_field("message", &self.0.get_message())?;
        state.serialize_field("extended_data", &self.0.get_extended_data())?;
        state.end()
    }
}

struct BatchStatusWrapper(ClientBatchStatus);
impl Serialize for BatchStatusWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("BatchStatusWrapper", 3)?;
        state.serialize_field("id", &self.0.get_batch_id())?;
        state.serialize_field("status", &self.0.get_status().descriptor().name())?;
        state.serialize_field::<Vec<InvalidTransactionWrapper>>(
            "invalid_transactions",
            &self
                .0
                .get_invalid_transactions()
                .iter()
                .map(|invalid_transaction| InvalidTransactionWrapper(invalid_transaction.clone()))
                .collect(),
        )?;
        state.end()
    }
}

#[post("/batches", format = "application/octet-stream", data = "<data>")]
pub fn submit_batches(data: Data, validator_url: State<String>) -> Result<JsonValue, ApiError> {
    let mut buffer = Vec::new();
    data.open().read_to_end(&mut buffer).unwrap();
    let batch_list: BatchList =
        protobuf::parse_from_bytes(&buffer).map_err(|err| ApiError::BadRequest(err.to_string()))?;
    let batch_ids: Vec<String> = batch_list
        .batches
        .iter()
        .map(|ref batch| batch.header_signature.clone())
        .collect();

    let mut batch_submit_request = ClientBatchSubmitRequest::new();
    batch_submit_request.set_batches(batch_list.batches);
    let response: ClientBatchSubmitResponse = send_request(
        validator_url,
        Message_MessageType::CLIENT_BATCH_SUBMIT_REQUEST,
        &batch_submit_request,
    )
    .map_err(|err| ApiError::InternalError(err.to_string()))?;

    match response.status {
        ClientBatchSubmitResponse_Status::OK => Ok(
            json!({ "link": "/batch_statuses?id=".to_string() + &batch_ids.join(",") }),
        ),
        ClientBatchSubmitResponse_Status::STATUS_UNSET => {
            Err(ApiError::InternalError("Validator error".to_string()))
        }
        ClientBatchSubmitResponse_Status::INTERNAL_ERROR => {
            Err(ApiError::InternalError("Validator error".to_string()))
        }
        ClientBatchSubmitResponse_Status::INVALID_BATCH => {
            Err(ApiError::BadRequest("Invalid batch".to_string()))
        }
        ClientBatchSubmitResponse_Status::QUEUE_FULL => Err(ApiError::TooManyRequests(
            "Validator queue full".to_string(),
        )),
    }
}

#[derive(FromForm)]
pub struct BatchStatusesParams {
    id: String,
    wait: Option<u32>,
}

#[get("/batch_statuses?<params..>")]
pub fn list_statuses(
    params: Form<BatchStatusesParams>,
    validator_url: State<String>,
) -> Result<JsonValue, ApiError> {
    let batch_ids: Vec<String> = params.id.split(',').map(|id| id.to_string()).collect();

    let mut batch_status_request = ClientBatchStatusRequest::new();
    batch_status_request.set_batch_ids(protobuf::RepeatedField::from_vec(batch_ids));
    if let Some(wait) = params.wait {
        batch_status_request.set_wait(true);
        batch_status_request.set_timeout(wait);
    }

    let response: ClientBatchStatusResponse = send_request(
        validator_url,
        Message_MessageType::CLIENT_BATCH_STATUS_REQUEST,
        &batch_status_request,
    )
    .map_err(|err| ApiError::InternalError(err.to_string()))?;

    match response.status {
        ClientBatchStatusResponse_Status::OK => {
            let batch_statuses: Vec<BatchStatusWrapper> = response
                .batch_statuses
                .into_vec()
                .iter()
                .map(|batch_status| BatchStatusWrapper(batch_status.clone()))
                .collect();

            Ok(json!({
                "data": batch_statuses,
                "link": "/batch_statuses?id=".to_string() + &params.id
            }))
        }
        ClientBatchStatusResponse_Status::STATUS_UNSET => {
            Err(ApiError::InternalError("Validator error".to_string()))
        }
        ClientBatchStatusResponse_Status::INTERNAL_ERROR => {
            Err(ApiError::InternalError("Validator error".to_string()))
        }
        ClientBatchStatusResponse_Status::NO_RESOURCE => {
            Err(ApiError::BadRequest("No resource".to_string()))
        }
        ClientBatchStatusResponse_Status::INVALID_ID => {
            Err(ApiError::BadRequest("Invalid ID".to_string()))
        }
    }
}

fn send_request<T, U>(
    validator_url: State<String>,
    msg_type: Message_MessageType,
    msg: &T,
) -> Result<U, String>
where
    T: protobuf::Message,
    U: protobuf::Message,
{
    let connection = ZmqMessageConnection::new(&validator_url);
    let (sender, _) = connection.create();
    let correlation_id = uuid::Uuid::new_v4()
        .to_simple().to_string();
    let msg_bytes = T::write_to_bytes(&msg).unwrap();
    let mut future = sender
        .send(msg_type, &correlation_id, &msg_bytes)
        .map_err(|err| err.to_string())?;
    let response_msg = future
        .get()
        .map_err(|err| format!("Unable to retrieve response from validator: {}", err))?;
    Ok(protobuf::parse_from_bytes(&response_msg.content).expect("Unable to parse protobuf"))
}
