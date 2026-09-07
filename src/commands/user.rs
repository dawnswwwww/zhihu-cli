use crate::cli::UserCommand;
use crate::client::ZhihuClient;
use crate::error::Result;
use crate::output::{dispatch_result, print_error};

pub async fn run(cmd: UserCommand) {
    if let Err(e) = dispatch_result(handle(cmd).await) {
        print_error(&e);
    }
}

async fn handle(cmd: UserCommand) -> Result<serde_json::Value> {
    handle_with_client(cmd, &ZhihuClient::new()?).await
}

/// Testable inner core: takes the client as a parameter so unit tests can
/// pass a mock. Public within the crate for tests.
pub(crate) async fn handle_with_client(cmd: UserCommand, client: &ZhihuClient) -> Result<serde_json::Value> {
    let req = build_request(&cmd);
    let query_refs: Vec<(&str, &str)> = req.query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    client
        .get_with_oauth(req.path, &query_refs, req.oauth_token.as_deref())
        .await
}

/// The fully-prepared request for a user-data command: HTTP path, query
/// parameters, and the optional OAuth token for querying another
/// authorized user.
#[derive(Debug, PartialEq)]
pub(crate) struct UserRequest {
    pub path: &'static str,
    pub query: Vec<(&'static str, String)>,
    pub oauth_token: Option<String>,
}

/// Validate inputs and assemble the (path, query, oauth token) for a
/// user-data command.
///
/// `Limit` is clamped to `[1, 50]` for contents/followees to match the
/// documented API maximum.
pub(crate) fn build_request(cmd: &UserCommand) -> UserRequest {
    match cmd {
        UserCommand::Contents {
            content_type,
            offset,
            limit,
            sort_field,
            sort_order,
            oauth_token,
        } => UserRequest {
            path: "/api/v1/user/contents",
            query: vec![
                ("ContentType", content_type.api_name().to_string()),
                ("Offset", offset.to_string()),
                ("Limit", (*limit).clamp(1, 50).to_string()),
                ("SortField", sort_field.api_name().to_string()),
                ("SortOrder", sort_order.api_name().to_string()),
            ],
            oauth_token: oauth_token.clone(),
        },
        UserCommand::Followees {
            offset,
            limit,
            oauth_token,
        } => UserRequest {
            path: "/api/v1/user/followees",
            query: vec![
                ("Offset", offset.to_string()),
                ("Limit", (*limit).clamp(1, 50).to_string()),
            ],
            oauth_token: oauth_token.clone(),
        },
        UserCommand::Collections { limit, oauth_token } => UserRequest {
            path: "/api/v1/user/collections",
            query: vec![("Limit", limit.to_string())],
            oauth_token: oauth_token.clone(),
        },
        UserCommand::Favlists { limit, oauth_token } => UserRequest {
            path: "/api/v1/user/favlists",
            query: vec![("Limit", limit.to_string())],
            oauth_token: oauth_token.clone(),
        },
        UserCommand::FavlistContents {
            favlist_url_token,
            offset,
            limit,
            oauth_token,
        } => UserRequest {
            path: "/api/v1/user/favlist_contents",
            query: vec![
                ("FavlistUrlToken", favlist_url_token.to_string()),
                ("Offset", offset.to_string()),
                ("Limit", limit.to_string()),
            ],
            oauth_token: oauth_token.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the user command group's pure parameter-assembly layer
    //! and the client-injected handler. The full binary path is exercised by
    //! `tests/cli.rs`.

    use crate::cli::{SortField, SortOrder, UserCommand, UserContentType};

    fn contents(
        content_type: UserContentType,
        offset: i64,
        limit: i64,
        sort_field: SortField,
        sort_order: SortOrder,
        oauth_token: Option<&str>,
    ) -> UserCommand {
        UserCommand::Contents {
            content_type,
            offset,
            limit,
            sort_field,
            sort_order,
            oauth_token: oauth_token.map(str::to_string),
        }
    }

    #[test]
    fn contents_request_assembles_full_query() {
        let cmd = contents(UserContentType::Answer, 20, 30, SortField::LikeCount, SortOrder::Asc, None);
        let req = super::build_request(&cmd);
        assert_eq!(req.path, "/api/v1/user/contents");
        assert_eq!(
            req.query,
            vec![
                ("ContentType", "answer".to_string()),
                ("Offset", "20".to_string()),
                ("Limit", "30".to_string()),
                ("SortField", "like_count".to_string()),
                ("SortOrder", "asc".to_string()),
            ]
        );
        assert!(req.oauth_token.is_none());
    }

    #[test]
    fn contents_request_defaults_use_api_defaults() {
        let cmd = contents(UserContentType::All, 0, 20, SortField::Ts, SortOrder::Desc, None);
        let req = super::build_request(&cmd);
        assert_eq!(
            req.query,
            vec![
                ("ContentType", "all".to_string()),
                ("Offset", "0".to_string()),
                ("Limit", "20".to_string()),
                ("SortField", "ts".to_string()),
                ("SortOrder", "desc".to_string()),
            ]
        );
    }

    #[test]
    fn contents_request_carries_oauth_token() {
        let cmd = contents(UserContentType::All, 0, 20, SortField::Ts, SortOrder::Desc, Some("tok"));
        let req = super::build_request(&cmd);
        assert_eq!(req.oauth_token.as_deref(), Some("tok"));
    }

    #[test]
    fn contents_request_limit_clamps_to_one_through_fifty() {
        let make = |limit: i64| contents(UserContentType::All, 0, limit, SortField::Ts, SortOrder::Desc, None);
        let assert_limit = |limit: i64, expected: &str| {
            let req = super::build_request(&make(limit));
            let pair = req.query.iter().find(|(k, _)| *k == "Limit").unwrap();
            assert_eq!(pair.1, expected, "limit {limit} should clamp to {expected}");
        };
        assert_limit(0, "1");
        assert_limit(-3, "1");
        assert_limit(1, "1");
        assert_limit(30, "30");
        assert_limit(50, "50");
        assert_limit(51, "50");
        assert_limit(1000, "50");
    }

    #[test]
    fn followees_request_assembles_query() {
        let cmd = UserCommand::Followees { offset: 40, limit: 10, oauth_token: None };
        let req = super::build_request(&cmd);
        assert_eq!(req.path, "/api/v1/user/followees");
        assert_eq!(
            req.query,
            vec![("Offset", "40".to_string()), ("Limit", "10".to_string())]
        );
    }

    #[test]
    fn followees_request_limit_clamps_to_one_through_fifty() {
        let cmd = UserCommand::Followees { offset: 0, limit: 99, oauth_token: None };
        let req = super::build_request(&cmd);
        let pair = req.query.iter().find(|(k, _)| *k == "Limit").unwrap();
        assert_eq!(pair.1, "50");
    }

    #[test]
    fn collections_request_assembles_query() {
        let cmd = UserCommand::Collections { limit: 7, oauth_token: None };
        let req = super::build_request(&cmd);
        assert_eq!(req.path, "/api/v1/user/collections");
        assert_eq!(req.query, vec![("Limit", "7".to_string())]);
    }

    #[test]
    fn favlists_request_assembles_query() {
        let cmd = UserCommand::Favlists { limit: 20, oauth_token: Some("tok".into()) };
        let req = super::build_request(&cmd);
        assert_eq!(req.path, "/api/v1/user/favlists");
        assert_eq!(req.query, vec![("Limit", "20".to_string())]);
        assert_eq!(req.oauth_token.as_deref(), Some("tok"));
    }

    #[test]
    fn favlist_contents_request_assembles_query() {
        let cmd = UserCommand::FavlistContents {
            favlist_url_token: 123456789,
            offset: 20,
            limit: 15,
            oauth_token: None,
        };
        let req = super::build_request(&cmd);
        assert_eq!(req.path, "/api/v1/user/favlist_contents");
        assert_eq!(
            req.query,
            vec![
                ("FavlistUrlToken", "123456789".to_string()),
                ("Offset", "20".to_string()),
                ("Limit", "15".to_string()),
            ]
        );
    }

    #[test]
    fn user_content_type_api_names() {
        assert_eq!(UserContentType::All.api_name(), "all");
        assert_eq!(UserContentType::Answer.api_name(), "answer");
        assert_eq!(UserContentType::Article.api_name(), "article");
        assert_eq!(UserContentType::Zvideo.api_name(), "zvideo");
        assert_eq!(UserContentType::Pin.api_name(), "pin");
        assert_eq!(UserContentType::Question.api_name(), "question");
    }

    #[test]
    fn sort_field_and_order_api_names() {
        assert_eq!(SortField::Ts.api_name(), "ts");
        assert_eq!(SortField::LikeCount.api_name(), "like_count");
        assert_eq!(SortOrder::Asc.api_name(), "asc");
        assert_eq!(SortOrder::Desc.api_name(), "desc");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_contents_sends_oauth_header() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/user/contents"))
            .and(query_param("ContentType", "all"))
            .and(wiremock::matchers::header("X-OAuth-Token", "tok-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": { "Items": [], "Paging": { "IsEnd": true, "Totals": 0 } }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = contents(UserContentType::All, 0, 20, SortField::Ts, SortOrder::Desc, Some("tok-1"));
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn handle_with_client_favlist_contents_calls_endpoint() {
        use serde_json::json;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/user/favlist_contents"))
            .and(query_param("FavlistUrlToken", "123456789"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "Code": 0,
                "Message": "success",
                "Data": { "Items": [], "Paging": { "IsEnd": true, "Totals": 0 } }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = crate::client::ZhihuClient::with_secret_and_base_url("fake".into(), server.uri());
        let cmd = UserCommand::FavlistContents {
            favlist_url_token: 123456789,
            offset: 0,
            limit: 20,
            oauth_token: None,
        };
        let result = super::handle_with_client(cmd, &client).await.unwrap();
        assert_eq!(result["Code"], 0);
    }
}
