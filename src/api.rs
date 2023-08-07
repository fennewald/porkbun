use crate::{Error, Status};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Deserialize)]
struct Response<T> {
    status: Status,
    message: Option<String>,
    #[serde(flatten)]
    payload: Option<T>,
}

impl<T> Into<Result<T, Error>> for Response<T> {
    fn into(self) -> Result<T, Error> {
        if self.status == Status::Success {
            self.payload.ok_or(Error::UnexpectedError)
        } else {
            Err(Error::ApiError(self.message.ok_or(Error::UnexpectedError)?))
        }
    }
}

fn url(path: &str) -> String {
    format!("https://porkbun.com/api/json/v3{}", path)
}

pub fn request<S: Serialize, R: DeserializeOwned>(endpoint: &str, payload: &S) -> Result<R, Error> {
    reqwest::blocking::Client::new()
        .post(url(endpoint))
        .json(payload)
        .send()?
        .json::<Response<R>>()?
        .into()
}
