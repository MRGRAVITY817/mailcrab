use {
    crate::{
        domain::{NewSubscriber, SubscriberEmail, SubscriberName},
        email_client::EmailClient,
    },
    actix_web::{
        web::{Data, Form},
        HttpResponse,
    },
    chrono::Utc,
    serde::Deserialize,
    sqlx::PgPool,
    uuid::Uuid,
};

#[derive(Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
    let email = SubscriberEmail::parse(form.email)?;
    let name = SubscriberName::parse(form.name)?;
    Ok(NewSubscriber { email, name })
}

#[tracing::instrument(
    name = "Adding a new subscriber", 
    skip(form, pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: Form<FormData>,
    pool: Data<PgPool>,
    email_client: Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber = match parse_subscriber(form.0) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    // Check if subscriber info is wrong
    if insert_subscriber(&pool, &new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    // Send subscription mail, and check if the server returns error
    if email_client
        .send_email(
            new_subscriber.email,
            "Welcome",
            "Welcome to mailcrab",
            "Welcome to maincrab",
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    // `insert_subscriber` only focuses on database logic
    // it knows nothing about actix-specific stuffs,
    // which forms nice segregation when moving to another web framework
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
