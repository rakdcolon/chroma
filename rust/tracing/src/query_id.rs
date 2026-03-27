use std::future::Future;

pub const QUERY_ID_HEADER_KEY: &str = "chroma-queryid";

tokio::task_local! {
    static QUERY_ID: String;
}

pub async fn with_query_id<F>(query_id: String, future: F) -> F::Output
where
    F: Future,
{
    QUERY_ID.scope(query_id, future).await
}

pub fn current_query_id() -> Option<String> {
    QUERY_ID.try_with(Clone::clone).ok()
}
