use ai_lib::transport::dyn_transport::DynHttpTransport;
use ai_lib::types::AiLibError;
use bytes::Bytes;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
// std::sync::Arc is referenced via type aliases later; keep direct path when used

pub struct MockTransport {
    pub response: serde_json::Value,
    pub fail: bool,
    pub non_json: bool,
}

impl MockTransport {
    pub fn new(response: serde_json::Value) -> Self {
        Self {
            response,
            fail: false,
            non_json: false,
        }
    }

    #[allow(dead_code)]
    pub fn failing(response: serde_json::Value) -> Self {
        Self {
            response,
            fail: true,
            non_json: false,
        }
    }

    #[allow(dead_code)]
    pub fn non_json(response: serde_json::Value) -> Self {
        Self {
            response,
            fail: false,
            non_json: true,
        }
    }
}

impl DynHttpTransport for MockTransport {
    fn get_json<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
    ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
        let resp = self.response.clone();
        let fail = self.fail;
        Box::pin(async move {
            if fail {
                Err(AiLibError::ProviderError(
                    "simulated transport failure".to_string(),
                ))
            } else {
                Ok(resp)
            }
        })
    }

    fn post_json<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        _body: serde_json::Value,
    ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
        let resp = self.response.clone();
        let fail = self.fail;
        Box::pin(async move {
            if fail {
                Err(AiLibError::ProviderError(
                    "simulated transport failure".to_string(),
                ))
            } else {
                Ok(resp)
            }
        })
    }

    fn post_stream<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        _body: serde_json::Value,
    ) -> futures::future::BoxFuture<
        'a,
        Result<Pin<Box<dyn Stream<Item = Result<Bytes, AiLibError>> + Send>>, AiLibError>,
    > {
        Box::pin(async move {
            Err(AiLibError::ProviderError(
                "stream not supported in mock".to_string(),
            ))
        })
    }

    fn upload_multipart<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        _field_name: &'a str,
        _file_name: &'a str,
        _bytes: Vec<u8>,
    ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
        let resp = self.response.clone();
        let fail = self.fail;
        let non_json = self.non_json;
        Box::pin(async move {
            if fail {
                return Err(AiLibError::ProviderError(
                    "simulated upload failure".to_string(),
                ));
            }
            if non_json {
                // return a JSON value that lacks url/id to simulate non-json body parse
                return Ok(resp);
            }
            Ok(resp)
        })
    }
}

#[allow(dead_code)]
pub type MockTransportRef = std::sync::Arc<MockTransport>;
