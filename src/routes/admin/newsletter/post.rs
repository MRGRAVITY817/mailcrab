use {
    crate::{
        authentication::UserId,
        domain::SubscriberEmail,
        email_client::EmailClient,
        idempotency::{save_response, try_processing, IdempotencyKey, NextAction},
        utils::e400,
        utils::{e500, see_other},
    },
    actix_web::{web, HttpResponse},
    actix_web_flash_messages::FlashMessage,
    anyhow::Context,
    sqlx::PgPool,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
    idempotency_key: String,
}

fn success_message() -> FlashMessage {
    FlashMessage::info("The newsletter issue has been published!")
}

/// Publish a newsletter issue to subscribed users
#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(form, pool, email_client, user_id),
    fields(user_id=%*user_id)
)]
pub async fn publish_issue(
    form: web::Form<FormData>,
    user_id: web::ReqData<UserId>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let FormData {
        title,
        text_content,
        html_content,
        idempotency_key,
    } = form.0;
    // Return early if we have a saved response in the database, since it's already been sent
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    let transaction = match try_processing(&pool, &idempotency_key, **user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            success_message().send();
            return Ok(saved_response);
        }
    };

    // Get all the confirmed subscribers
    let subscribers = get_confirmed_subscribers(&pool).await.map_err(e500)?;
    for subscriber in subscribers {
        match subscriber {
            // Send issue to all of confirmed subscribers
            Ok(subscriber) => {
                email_client
                    .send_email(&subscriber.email, &title, &html_content, &text_content)
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })
                    .map_err(e500)?;
            }
            // If subscriber's email address has a problem, omit error
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    error.message =  %error,
                    "Skipping a confirmed subscriber. Their stored contact details are invalid",
                );
            }
        }
    }

    // Send a flash message that we've published all the newsletters.
    success_message().send();
    let response = see_other("/admin/newsletter");
    let response = save_response(&idempotency_key, **user_id, response, transaction)
        .await
        .map_err(e500)?;

    Ok(response)
}

/// Subscriber whose status is `confirmed`
struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

/// Get confirmed subscribers
#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email 
        FROM subscriptions 
        WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(error) => Err(anyhow::anyhow!(error)),
    })
    .collect();

    Ok(confirmed_subscribers)
}
