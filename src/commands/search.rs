use crate::cli::SearchCommand;
use crate::client::ZhihuClient;
use crate::error::Result;
use crate::output::{print_error, print_json};

pub async fn run(cmd: SearchCommand) {
    match handle(cmd).await {
        Ok(value) => print_json(&value),
        Err(e) => print_error(&e),
    }
}

async fn handle(cmd: SearchCommand) -> Result<serde_json::Value> {
    let client = ZhihuClient::new()?;
    match cmd {
        SearchCommand::Zhihu { query, count } => {
            let count = count.clamp(1, 10).to_string();
            client
                .get(
                    "/api/v1/content/zhihu_search",
                    &[("Query", &query), ("Count", &count)],
                )
                .await
        }
        SearchCommand::Global {
            query,
            count,
            filter,
            db,
        } => {
            let count = count.clamp(1, 20).to_string();
            let mut params: Vec<(&str, &str)> = vec![("Query", &query), ("Count", &count)];
            let db_str = db.api_name().to_string();
            params.push(("SearchDB", &db_str));
            if let Some(filter) = &filter {
                params.push(("Filter", filter.as_str()));
            }
            client.get("/api/v1/content/global_search", &params).await
        }
    }
}
