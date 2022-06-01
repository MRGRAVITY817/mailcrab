use {
    actix_session::{Session, SessionExt},
    actix_web::FromRequest,
    std::future::{ready, Ready},
    uuid::Uuid,
};

pub struct TypedSession(Session);

// TypedSession will work as a actix request handler
impl FromRequest for TypedSession {
    type Error = <Session as FromRequest>::Error;
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";

    pub fn renew(&self) {
        self.0.renew()
    }

    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), serde_json::Error> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }

    pub fn get_user_id(&self) -> Result<Option<Uuid>, serde_json::Error> {
        self.0.get(Self::USER_ID_KEY)
    }
}
