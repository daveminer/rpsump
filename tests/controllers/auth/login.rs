use crate::helpers::spawn_app;
use uuid::Uuid;

#[tokio::test]
async fn login_failed_username_not_found() {
    // Arrange
    let app = spawn_app().await;
    let email = "test_acct@test.local";
    let password = "testing_*Password";

    // Act
    let response = app
        .post_login(&serde_json::json!({
            "email": email,
            "password": password,
        }))
        .await;

    // Assert
    println!(response);
    //assert_is_redirect_to(&response, "/login");
}
