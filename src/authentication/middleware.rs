use {
    crate::{
        session_state::TypedSession,
        utils::{e500, see_other},
    },
    actix_web::{
        body::MessageBody,
        dev::{ServiceRequest, ServiceResponse},
        error::InternalError,
        FromRequest, HttpMessage,
    },
    actix_web_lab::middleware::Next,
    uuid::Uuid,
};

/// User id that should be verified by session
#[derive(Copy, Clone, Debug)]
pub struct UserId(Uuid);

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Deref for UserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A middleware that reject anonymous user by checking user id from session
pub async fn reject_anonymous_users(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let session = {
        let (http_request, payload) = req.parts_mut();
        TypedSession::from_request(http_request, payload).await
    }?;

    match session.get_user_id().map_err(e500)? {
        // Once user id is found, forward it to app state
        Some(user_id) => {
            req.extensions_mut().insert(UserId(user_id));
            next.call(req).await
        }
        // If session cannot retrieve given user id, reject it.
        None => {
            let response = see_other("/login");
            let e = anyhow::anyhow!("The user has not logged in");

            Err(InternalError::from_response(e, response).into())
        }
    }
}
