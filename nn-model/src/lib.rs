#![allow(unused)]

use async_trait::async_trait;
use reqwest::{multipart, Client, Method, Url};

use serde::{self, de, Deserialize, Serialize};

use thiserror::Error;

use std::{borrow::Cow, collections::VecDeque, io::Bytes};

use tokio::fs;

#[derive(Error, Debug)]
#[error("embedding server error")]
pub struct EmbeddingError;

#[async_trait]
pub trait Embedding
    where
        Self: IntoIterator<Item = f64> + Sized
{
    async fn from_image(filepath: &str) -> Result<Self, EmbeddingError>;

    fn distance(&self, other: &Self) -> f64;
}

#[async_trait]
impl Embedding for Vec<f64> {
    async fn from_image(filepath: &str) -> Result<Self, EmbeddingError> {
        let client = Client::new();

        let embedding = client
            .request(Method::POST, "http://127.0.0.1:5000/embed")
            .multipart(get_image_form_from_file(filepath).await?)
            .send()
            .await.map_err(|_|EmbeddingError)?
            .json()
            .await.map_err(|_|EmbeddingError)?;

        Ok(embedding)
    }


    fn distance(&self, other: &Self) -> f64 {
        self.iter()
            .zip(other.iter())
            .map(|(x1, x2)| (x1 - x2).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

async fn get_image_form_from_file(filepath: &str) -> Result<multipart::Form, EmbeddingError> {
    let file = {
        let file = fs::read(filepath).await
            .map_err(|_|EmbeddingError)?;
        multipart::Part::bytes(file)
            .file_name(filepath.to_owned())
            .mime_str("image/png")
            .map_err(|_|EmbeddingError)?
    };
    Ok(multipart::Form::new().part("image", file))
}


#[cfg(test)]
mod tests {
    use std::{
        path::Path,
        process::{Command, Stdio},
        time::Duration,
    };

    use tokio::time;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn same_image_has_zero_distance() {
        todo!("reimplement the test");

        let project_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

        let mut server_cmd = Command::new("python3")
            .args([format!("{}/model/embbed.py", project_dir.to_str().unwrap()).as_str()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let image_path = project_dir.join("image.png").to_str().unwrap().to_owned();

        let Ok((emb1, emb2)) = time::timeout(Duration::from_secs(6), async {
            loop {
                time::sleep(Duration::from_secs(1)).await;
                let (emb1, emb2) = (
                    Vec::from_image(&image_path).await,
                    Vec::from_image(&image_path).await,
                );
                if let (Ok(emb1), Ok(emb2)) = (emb1, emb2) {
                    break (emb1, emb2); 
                }
            }
        }).await else {
            server_cmd.kill().unwrap();
            panic!("flask server timed out");
        };

        assert_eq!(0.0, emb1.distance(&emb2));

        server_cmd.kill().unwrap();
    }
}
