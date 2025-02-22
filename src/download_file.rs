use crate::file_analyser::FileAnalyser;
use crate::types::urls::url::Url;
use crate::url_utils::find_architecture;
use color_eyre::eyre::{eyre, Result, WrapErr};
use futures_util::{stream, StreamExt, TryStreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::Itertools;
use reqwest::header::{HeaderValue, CONTENT_DISPOSITION, LAST_MODIFIED};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::future::Future;
use time::format_description::well_known::Rfc2822;
use time::{Date, OffsetDateTime};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

async fn download_file(
    client: &Client,
    url: Url,
    multi_progress: &MultiProgress,
) -> Result<DownloadedFile> {
    let res = client
        .get(url.as_str())
        .send()
        .await
        .wrap_err_with(|| format!("Failed to GET from '{url}'"))?;

    let content_disposition = res.headers().get(CONTENT_DISPOSITION);
    let file_name = get_file_name(&url, content_disposition);
    let total_size = res
        .content_length()
        .ok_or_else(|| eyre!("Failed to get content length from '{url}'"))?;

    let last_modified = res
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|last_modified| last_modified.to_str().ok())
        .and_then(|last_modified| OffsetDateTime::parse(last_modified, &Rfc2822).ok())
        .map(OffsetDateTime::date);

    let pb = multi_progress.add(ProgressBar::new(total_size)
        .with_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
            .progress_chars("#>-")
        )
        .with_message(format!("Downloading {url}"))
    );

    // Download chunks
    let temp_file = tempfile::tempfile()?;
    let mut file = tokio::fs::File::from_std(temp_file.try_clone()?);
    let mut downloaded = 0;
    let mut stream = res.bytes_stream();

    let mut hasher = Sha256::new();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        let write = file.write_all(&chunk);
        hasher.update(&chunk); // Hash file as it's downloading
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
        write.await?;
    }
    pb.finish_and_clear();

    Ok(DownloadedFile {
        url,
        file: temp_file,
        sha_256: base16ct::upper::encode_string(&hasher.finalize()),
        file_name,
        last_modified,
    })
}

fn get_file_name(url: &Url, content_disposition: Option<&HeaderValue>) -> String {
    if let Some(content_disposition) = content_disposition.and_then(|value| value.to_str().ok()) {
        let mut sections = content_disposition.split(';');
        let _disposition = sections.next();
        for section in sections {
            let mut parts = section.splitn(2, '=');

            let key = parts.next().map(str::trim);
            let value = parts.next().map(str::trim);
            if let (Some(key), Some(value)) = (key, value) {
                if key.starts_with("filename") {
                    return value.trim_matches('"').to_owned();
                }
            }
        }
    }
    url.path_segments()
        .and_then(Iterator::last)
        .map_or_else(|| Uuid::new_v4().to_string(), str::to_owned)
}

pub fn download_urls<'a>(
    client: &'a Client,
    urls: Vec<Url>,
    multi_progress: &'a MultiProgress,
) -> impl Iterator<Item = impl Future<Output = Result<DownloadedFile>> + 'a> {
    urls.into_iter()
        .unique()
        .map(|url| download_file(client, url, multi_progress))
}

pub struct DownloadedFile {
    pub url: Url,
    pub file: File,
    pub sha_256: String,
    pub file_name: String,
    pub last_modified: Option<Date>,
}

pub async fn process_files<'a>(
    files: Vec<DownloadedFile>,
) -> Result<HashMap<Url, FileAnalyser<'a>>> {
    stream::iter(files.into_iter().map(
        |DownloadedFile {
             url,
             file,
             sha_256,
             file_name,
             last_modified,
         }| async move {
            let mut file_analyser = FileAnalyser::new(&file, Cow::Owned(file_name), false)?;
            file_analyser.architecture =
                find_architecture(url.as_str()).unwrap_or(file_analyser.architecture);
            file_analyser.installer_sha_256 = sha_256;
            file_analyser.last_modified = last_modified;
            Ok((url, file_analyser))
        },
    ))
    .buffered(num_cpus::get())
    .try_collect::<HashMap<_, _>>()
    .await
}
