use crate::cli::HotArgs;

#[derive(Debug, PartialEq)]
pub(crate) struct HotRequest {
    pub path: &'static str,
    pub query: Vec<(&'static str, String)>,
}

pub(crate) fn build_request(args: &HotArgs) -> HotRequest {
    HotRequest {
        path: "/api/v1/content/hot_list",
        query: vec![("Limit", args.limit.clamp(1, 30).to_string())],
    }
}

#[cfg(test)]
mod tests {
    use super::build_request;
    use crate::cli::HotArgs;

    fn pairs(req: &super::HotRequest) -> Vec<(&str, &str)> {
        req.query.iter().map(|(k, v)| (*k, v.as_str())).collect()
    }

    #[test]
    fn hot_request_uses_hot_list_path() {
        let args = HotArgs { limit: 10 };
        let req = build_request(&args);
        assert_eq!(req.path, "/api/v1/content/hot_list");
    }

    #[test]
    fn hot_request_limit_defaults_to_thirty() {
        let args = HotArgs { limit: 30 };
        let req = build_request(&args);
        let limit = pairs(&req).into_iter().find(|(k, _)| *k == "Limit").unwrap();
        assert_eq!(limit.1, "30");
    }

    #[test]
    fn hot_request_limit_clamps_to_one_through_thirty() {
        let make = |limit: i32| HotArgs { limit };
        let assert_limit = |limit: i32, expected: &str| {
            let req = build_request(&make(limit));
            let pair = pairs(&req).into_iter().find(|(k, _)| *k == "Limit").unwrap();
            assert_eq!(pair.1, expected, "limit {limit} should clamp to {expected}");
        };
        assert_limit(0, "1");
        assert_limit(-5, "1");
        assert_limit(1, "1");
        assert_limit(15, "15");
        assert_limit(30, "30");
        assert_limit(31, "30");
        assert_limit(1000, "30");
    }
}
