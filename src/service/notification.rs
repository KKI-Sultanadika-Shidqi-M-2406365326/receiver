use std::thread;

use rocket::http::Status;
use rocket::serde::json::to_string;
use rocket::tokio;

use bambangshop_receiver::{
    APP_CONFIG,
    REQWEST_CLIENT,
    Result,
    compose_error_response,
};

use crate::model::notification::Notification;
use crate::model::subscriber::SubscriberRequest;
use crate::repository::notification::NotificationRepository;

pub struct NotificationService;

impl NotificationService {

    pub fn subscribe(product_type: &str) -> Result<SubscriberRequest> {
        let product_type_clone: String = String::from(product_type);

        thread::spawn(move || {
            Self::subscribe_request(product_type_clone)
        })
        .join()
        .unwrap()
    }

    pub fn unsubscribe(product_type: &str) -> Result<SubscriberRequest> {
        let product_type_clone: String = String::from(product_type);

        thread::spawn(move || {
            Self::unsubscribe_request(product_type_clone)
        })
        .join()
        .unwrap()
    }

    pub fn receive_notification(payload: Notification) -> Result<Notification> {
        let result = NotificationRepository::add(payload);
        Ok(result)
    }

    pub fn list_messages() -> Result<Vec<String>> {
        Ok(NotificationRepository::list_all_as_string())
    }

    #[tokio::main]
    async fn subscribe_request(product_type: String) -> Result<SubscriberRequest> {
        let product_type_upper = product_type.to_uppercase();

        let notification_receiver_url = format!(
            "{}/receive",
            APP_CONFIG.get_instance_root_url()
        );

        let payload = SubscriberRequest {
            name: APP_CONFIG.get_instance_name().to_string(),
            url: notification_receiver_url,
        };

        let request_url = format!(
            "{}/notification/subscribe/{}",
            APP_CONFIG.get_publisher_root_url(),
            product_type_upper
        );

        let request = REQWEST_CLIENT
            .post(request_url.clone())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(to_string(&payload).unwrap())
            .send()
            .await;

        warn!("Sent subscribe request to: {}", request_url);

        match request {
            Ok(response) => match response.json::<SubscriberRequest>().await {
                Ok(data) => Ok(data),
                Err(err) => Err(compose_error_response(
                    Status::NotAcceptable,
                    err.to_string(),
                )),
            },
            Err(err) => Err(compose_error_response(
                Status::NotFound,
                err.to_string(),
            )),
        }
    }

    #[tokio::main]
    async fn unsubscribe_request(product_type: String) -> Result<SubscriberRequest> {
        let product_type_upper = product_type.to_uppercase();

        let notification_receiver_url = format!(
            "{}/receive",
            APP_CONFIG.get_instance_root_url()
        );

        let request_url = format!(
            "{}/notification/unsubscribe/{}?url={}",
            APP_CONFIG.get_publisher_root_url(),
            product_type_upper,
            notification_receiver_url
        );

        let request = REQWEST_CLIENT
            .post(request_url.clone())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .send()
            .await;

        warn!("Sent unsubscribe request to: {}", request_url);

        match request {
            Ok(response) => match response.json::<SubscriberRequest>().await {
                Ok(data) => Ok(data),
                Err(_) => Err(compose_error_response(
                    Status::NotFound,
                    String::from("Already unsubscribed to the topic."),
                )),
            },
            Err(err) => Err(compose_error_response(
                Status::NotFound,
                err.to_string(),
            )),
        }
    }
}
