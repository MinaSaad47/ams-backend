#![allow(unused)]

use async_trait::async_trait;
use reqwest::{multipart, Client, Method, Url};

use serde::{self, de, Deserialize, Serialize};

use thiserror::Error;
use uuid::Uuid;

use std::{borrow::Cow, collections::VecDeque, io::Bytes, str::FromStr};

use tokio::fs;

#[derive(Error, Debug)]
#[error("face recognition error")]
pub struct FaceRecognitionError;

#[async_trait]
pub trait Embedding
where
    Self: IntoIterator<Item = f64> + Sized,
{
    fn distance(&self, other: &Self) -> f64;
}

#[async_trait]
impl Embedding for Vec<f64> {
    fn distance(&self, other: &Self) -> f64 {
        self.iter()
            .zip(other.iter())
            .map(|(x1, x2)| (x1 - x2).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

pub struct FaceRecognizer {
    client: Client,
    classify_url: Url,
    embed_url: Url,
    upload_classifier_url: Url,
}

impl FaceRecognizer {
    pub fn new(base_url: &str) -> Self {
        let client = Client::new();

        Self {
            client,
            classify_url: Url::from_str(base_url)
                .and_then(|url| url.join("classify"))
                .unwrap(),
            embed_url: Url::from_str(base_url)
                .and_then(|url| url.join("embed"))
                .unwrap(),
            upload_classifier_url: Url::from_str(base_url)
                .and_then(|url| url.join("upload_classifier"))
                .unwrap(),
        }
    }

    pub async fn embed(&self, image: &[u8]) -> Result<Vec<f64>, FaceRecognitionError> {
        let part = multipart::Part::bytes(image.to_owned()).file_name("image");
        let multipart = multipart::Form::new().part("image", part);

        let embedding = self
            .client
            .request(Method::POST, self.embed_url.as_ref())
            .multipart(multipart)
            .send()
            .await
            .map_err(|_| FaceRecognitionError)?
            .json()
            .await
            .map_err(|_| FaceRecognitionError)?;

        Ok(embedding)
    }

    pub async fn classify(&self, image: &[u8]) -> Result<Uuid, FaceRecognitionError> {
        let part = multipart::Part::bytes(image.to_owned()).file_name("image");
        let multipart = multipart::Form::new().part("image", part);

        let class: Uuid = self
            .client
            .request(Method::POST, self.classify_url.as_ref())
            .multipart(multipart)
            .send()
            .await
            .map_err(|_| FaceRecognitionError)?
            .json()
            .await
            .map_err(|_| FaceRecognitionError)?;

        Ok(class)
    }

    pub async fn upload_classifier(
        &self,
        classifier: &[u8],
    ) -> Result<String, FaceRecognitionError> {
        let part = multipart::Part::bytes(classifier.to_owned()).file_name("classifier");
        let multipart = multipart::Form::new().part("model", part);

        let embedding = self
            .client
            .request(Method::POST, self.upload_classifier_url.as_ref())
            .multipart(multipart)
            .send()
            .await
            .map_err(|_| FaceRecognitionError)?
            .text()
            .await
            .map_err(|_| FaceRecognitionError)?;

        Ok(embedding)
    }
}
