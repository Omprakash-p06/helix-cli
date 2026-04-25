use crate::types::{PermissionRequest, PermissionResponse, PermissionRequester};
use inquire::Confirm;
use tokio::task;

pub struct InquirePermissionRequester;

#[async_trait::async_trait]
impl PermissionRequester for InquirePermissionRequester {
    async fn request_permission(&self, request: PermissionRequest) -> PermissionResponse {
        let prompt = format!(
            "Tool '{}' wants to execute with arguments:\n{}\nReason: {}\nAllow execution?",
            request.tool_name,
            serde_json::to_string_pretty(&request.arguments).unwrap_or_else(|_| "Error serializing arguments".to_string()),
            request.reason
        );

        let res = task::spawn_blocking(move || {
            Confirm::new(&prompt)
                .with_default(false)
                .prompt()
        })
        .await;

        match res {
            Ok(Ok(true)) => PermissionResponse::Allow,
            _ => PermissionResponse::Deny,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate;
    use mockall::mock;

    mock! {
        pub Requester {}
        #[async_trait::async_trait]
        impl PermissionRequester for Requester {
            async fn request_permission(&self, request: PermissionRequest) -> PermissionResponse;
        }
    }

    #[tokio::test]
    async fn test_permission_requester_mock() {
        let mut mock = MockRequester::new();
        let request = PermissionRequest {
            tool_name: "test_tool".to_string(),
            arguments: serde_json::json!({"arg1": "val1"}),
            reason: "testing".to_string(),
        };

        let request_clone = request.clone();
        mock.expect_request_permission()
            .with(predicate::eq(request_clone))
            .times(1)
            .returning(|_| PermissionResponse::Allow);

        let response = mock.request_permission(request).await;
        assert_eq!(response, PermissionResponse::Allow);
    }
}
