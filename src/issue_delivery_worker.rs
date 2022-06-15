use sqlx::{PgPool, Transaction, Postgres};
use tracing::{Span, field::display};
use uuid::Uuid;

use crate::email_client::EmailClient;

#[tracing::instrument(
    skip_all, 
    fields(
        newsletter_issue_id=tracing::field::Empty, 
        subscriber_email=tracing::field::Empty
    ), 
    err
)]
async fn try_execute_task(pool: &PgPool, email_client: &EmailClient) -> Result<(), anyhow::Error> {
    if let Some((transaction, issue_id, email)) = dequeue_task(pool).await? {
        // `Span::current` will send records to fields only when this block is being executed
        Span::current()
            .record("newsletter_issue_id", &display(issue_id))
            .record("subscriber_email", &display(&email));

        // TODO: send email
        delete_task(transaction, issue_id, &email).await?;
    }

    Ok(())
}

type PgTransaction = Transaction<'static, Postgres>;

#[tracing::instrument(skip_all)]
async fn dequeue_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String)>, anyhow::Error> {
    let mut transaction = pool.begin().await?;
    let r = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, subscriber_email
        FROM issue_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#
    )
    .fetch_optional(&mut transaction)
    .await?;

    if let Some(r) = r {
        Ok(Some((
            transaction,
            r.newsletter_issue_id,
            r.subscriber_email,
        )))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip_all)]
async fn delete_task(mut transaction: PgTransaction, issue_id: Uuid, email: &str) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        DELETE FROM issue_delivery_queue
        WHERE
            newsletter_issue_id = $1 AND
            subscriber_email = $2
        "#, issue_id, email
        ).execute(&mut transaction).await?;
    transaction.commit().await?;

    Ok(())
}
