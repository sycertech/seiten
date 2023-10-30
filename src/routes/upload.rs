use crate::config::CONFIG;
use anyhow::Result;
use axum::{
    body::Bytes,
    extract::Path,
    response::{IntoResponse, Response},
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use flate2::bufread::GzDecoder;
use gpgme::{Context, Data, Protocol};
use http::StatusCode;
use std::io::Cursor;
use tar::Archive;

#[derive(TryFromMultipart)]
pub struct UploadAssetRequest {
    /// The archive itself
    #[form_data(limit = "50MiB")]
    archive: FieldData<Bytes>,

    /// The signature of the archive
    signature: Bytes,
}

#[axum::debug_handler]
pub async fn upload(
    Path(out_dir): Path<String>,
    TypedMultipart(UploadAssetRequest { archive, signature }): TypedMultipart<UploadAssetRequest>,
) -> Response {
    // if the accepted directories is empty, allow all
    if !CONFIG.directories.contains(&out_dir) && !CONFIG.directories.is_empty() {
        return (
            StatusCode::FORBIDDEN,
            format!("`{out_dir}` is not an accepted directory to unpack files."),
        )
            .into_response();
    }

    if archive
        .metadata
        .file_name
        .map(|name| !name.ends_with(".tar.gz"))
        .unwrap_or(false)
    {
        return (StatusCode::BAD_REQUEST, "File must be a tar.gz file").into_response();
    }

    // create gpg context
    let mut ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut key_data = Data::from_bytes(CONFIG.gpg_public_key.as_bytes()).unwrap();
    ctx.import(&mut key_data).unwrap();

    let mut signed_data = Data::from_bytes(&archive.contents).unwrap();
    let mut signature_data = Data::from_bytes(&signature).unwrap();

    // validate GPG signature
    if ctx
        .verify_detached(&mut signature_data, &mut signed_data)
        .is_err()
    {
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
    }

    // decode tar.gz file
    let decoder = GzDecoder::new(Cursor::new(&archive.contents));
    let mut archive = Archive::new(decoder);

    archive.unpack(format!("/content/{}", out_dir)).unwrap();

    (StatusCode::ACCEPTED, "File uploaded and validated").into_response()
}
