use super::IdempotencyKey;
use actix_web::{body::to_bytes, http::StatusCode, HttpResponse};
use sqlx::{postgres::PgHasArrayType, PgPool};
use uuid::Uuid;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub enum NextAction {
    StartProcessing,
    ReturnSavedResponse(HttpResponse),
}

/// Try process the row insertion in `idempotency` table, and log the conflict
pub async fn try_processing(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<NextAction, anyhow::Error> {
    let n_inserted_rows = sqlx::query!(
        r#"
        INSERT INTO idempotency (
            user_id,
            idempotency_key,
            created_at
        )
        VALUES ($1, $2, now())
        ON CONFLICT DO NOTHING
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .execute(pool)
    .await?
    .rows_affected();

    if n_inserted_rows > 0 {
        Ok(NextAction::StartProcessing)
    } else {
        let saved_response = get_saved_response(pool, idempotency_key, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("We expected a saved response, we didn't find it"))?;

        Ok(NextAction::ReturnSavedResponse(saved_response))
    }
}

/// Get saved HttpResponse for each publication
pub async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
        SELECT
        response_status_code AS "response_status_code!",
        response_headers AS "response_headers:Vec<HeaderPairRecord>",
        response_body AS "response_body!"
        FROM idempotency
        WHERE
        user_id = $1 AND
        idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    if let Some(r) = saved_response {
        let status_code = StatusCode::from_u16(r.response_status_code.try_into()?)?;
        let mut response = HttpResponse::build(status_code);
        for HeaderPairRecord { name, value } in r.response_headers {
            response.append_header((name, value));
        }
        Ok(Some(response.body(r.response_body)))
    } else {
        Ok(None)
    }
}

/// Save HttpResponse to idempotency table
pub async fn save_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
    http_response: HttpResponse,
) -> Result<HttpResponse, anyhow::Error> {
    let (response_head, body) = http_response.into_parts();
    // Since `MessageBody::Error` is not `Send + Sync` type,
    // we need to manually map the error
    let body = to_bytes(body).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    let status_code = response_head.status().as_u16() as i16;
    let headers = response_head
        .headers()
        .iter()
        .map(|(name, value)| {
            let name = name.as_str().to_owned();
            let value = value.as_bytes().to_owned();
            HeaderPairRecord { name, value }
        })
        .collect::<Vec<_>>();

    sqlx::query_unchecked!(
        r#"
        INSERT INTO idempotency (
            user_id,
            idempotency_key,
            response_status_code,
            response_headers,
            response_body,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, now())
        "#,
        user_id,
        idempotency_key.as_ref(),
        status_code,
        headers,
        body.as_ref()
    )
    .execute(pool)
    .await?;

    let http_response = response_head.set_body(body).map_into_boxed_body();
    Ok(http_response)
}

/// Provide information to sqlx about our array type `_header_pair`
impl PgHasArrayType for HeaderPairRecord {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_header_pair")
    }
}
