use actix_web::{error::ErrorInternalServerError, web::BytesMut};
use protobuf::Message;

pub(crate) fn grpc_err_to_actix_err(err: grpcio::Error) -> actix_web::Error {
    actix_web::Error::from(ErrorInternalServerError(err))
}

pub(crate) fn pb_to_bytes_mut(message: Message) -> BytesMut {
    let ref mut raw_data = vec![];
    let mut bytes = BytesMut::new();
    message.write_to_vec(raw_data);
    bytes.extend_from_slice(raw_data);

    bytes
}
