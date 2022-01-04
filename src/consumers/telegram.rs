use std::{fmt::Debug, ops::Deref, sync::Arc};

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use futures::{stream::FuturesUnordered, Stream, StreamExt};
use log::{debug, info, warn};
use telegram_bot_raw::{self as tg, ChatId, ChatRef, SendMessage};
use tg::GetMe;
use tokio::task::JoinHandle;

use crate::{get_client, Event, TelegramConfig, RX};

pub fn run_telegram(mut rx: RX, config: TelegramConfig) -> JoinHandle<()> {
    tokio::spawn(async move {
        let config = Arc::new(config);
        let chats = config.chats.iter().map(|x| ChatRef::Id(ChatId::new(*x)));

        let me = send(GetMe, config.clone()).await;

        match me {
            Ok(me) => info!("Logged in as {}", me.username.unwrap_or(me.first_name)),
            Err(e) => {
                warn!("Failed to authenticate: {}", e);
                return;
            }
        }

        while let Ok(res) = rx.recv().await {
            match res {
                Event::Feed {
                    link,
                    title,
                    content,
                    entry_id,
                    ..
                } => {
                    info!("New Feed event: {}", entry_id);
                    let (max_len, _) = config.content_max_length.overflowing_sub(1);

                    let content = match content {
                        Some(content) => {
                            if content.len() > max_len {
                                let bytes = content
                                    .chars()
                                    .take(max_len)
                                    .chain("...\n\n".chars())
                                    .map(|x| x as u8)
                                    .collect::<Vec<_>>();

                                html2text::from_read(bytes.deref(), 200) + "\n"
                            } else {
                                html2text::from_read(content.as_bytes(), 200) + "\n"
                            }
                        }
                        None => String::new(),
                    };

                    let content = html_escape::encode_safe(&content);

                    debug!("Content: {}", content);

                    let link = if let Some(link) = link {
                        format!("[ <a href=\"{}\">Feed</a> ]", link)
                    } else {
                        "[ Feed ]".to_owned()
                    };

                    let msg = format!(
                        "<b>{}  {}</b>\n\n{}",
                        link,
                        title.unwrap_or_default(),
                        content,
                    );

                    let mut stream = send_to_all(
                        chats.clone().map(|chat| {
                            let mut msg = SendMessage::new(chat, &msg);
                            msg.parse_mode(tg::ParseMode::Html);
                            msg
                        }),
                        config.clone(),
                    );
                    while let Some(res) = stream.next().await {
                        match res {
                            Ok(_) => info!("Message sent"),
                            Err(e) => warn!("{:?}", e),
                        }
                    }
                }
            }
        }
    })
}

fn send_to_all<Req: tg::Request>(
    reqs: impl Iterator<Item = Req>,
    config: Arc<TelegramConfig>,
) -> impl Stream<Item = Result<<<Req as tg::Request>::Response as tg::ResponseType>::Type>>
where
    <<Req as tg::Request>::Response as tg::ResponseType>::Type: Debug,
{
    reqs.map(|req| send(req, config.clone()))
        .collect::<FuturesUnordered<_>>()
}

async fn send<Req: tg::Request>(
    req: Req,
    config: Arc<TelegramConfig>,
) -> Result<<<Req as tg::Request>::Response as tg::ResponseType>::Type> {
    let res = tg_raw_to_reqwest(
        req.serialize().wrap_err("Failed to serialize request")?,
        &config.api_token,
    )
    .await?;
    let status = res.status();

    if !status.is_success() {
        let text = res.text().await.wrap_err("Decode failed")?;
        Err(eyre!("{}", text).wrap_err(format!(
            "Unsuccessful response from server (Code: {})",
            status
        )))
    } else {
        reqwest_res_to_tg_raw::<Req::Response>(res).await
    }
}

async fn tg_raw_to_reqwest(tg_req: tg::HttpRequest, token: &str) -> Result<reqwest::Response> {
    let tg::HttpRequest { url, method, body } = tg_req;
    let method = match method {
        tg::Method::Get => reqwest::Method::GET,
        tg::Method::Post => reqwest::Method::POST,
    };
    let url = url.url(token);
    debug!("New request to {}", url);
    debug!("{}: {}", method, body);
    let req = get_client().request(method, url);

    if let tg::Body::Json(content) = body {
        debug!("JSON body: {}", content);
        req.body(content)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
    } else {
        req
    }
    .send()
    .await
    .wrap_err("Failed to request telegram API")
}

async fn reqwest_res_to_tg_raw<R: tg::ResponseType>(res: reqwest::Response) -> Result<R::Type> {
    let bytes = res.bytes().await?.to_vec();
    let tg_raw_res = tg::HttpResponse { body: Some(bytes) };
    R::deserialize(tg_raw_res).wrap_err("Failed to parse result")
}
