pub mod prelude;

use hyper::body::Bytes;
use serde::de::DeserializeOwned;

pub struct Wrapper<T>(pub T);

impl<ResBody> TryFrom<Bytes> for Wrapper<ResBody>
where
    ResBody: DeserializeOwned,
{
    type Error = serde_json::Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value).map(|v| Wrapper(v))
    }
}
