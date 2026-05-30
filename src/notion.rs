use crate::structs::NotionRowData;
use chrono::{TimeZone, Utc};
use notion_client::endpoints::Client as NotionClient;
use notion_client::endpoints::pages::create::request::CreateAPageRequestBuilder;
use notion_client::objects::page::{DatePropertyValue, PageProperty, SelectPropertyValue};
use notion_client::objects::parent::Parent;
use notion_client::objects::property::DateOrDateTime;
use notion_client::objects::rich_text::{RichText, Text};
use reqwest::ClientBuilder;
use serde_json::Number;
use std::collections::BTreeMap;
use thiserror::Error;
use tracing::info;

/// Errors specific to the NotionWriter.
#[derive(Error, Debug)]
pub enum NotionWriterError {
    #[error("Notion client error: {0}")]
    ClientError(String),
    #[error("Notion API request failed: {0}")]
    RequestError(#[from] notion_client::NotionClientError),
}

/// Writer for logging trade records to the Notion Database.
pub struct NotionWriter {
    client: NotionClient,
    database_id: String,
}

impl NotionWriter {
    /// Creates a new NotionWriter instance.
    pub fn new(token: String, database_id: String) -> Result<Self, NotionWriterError> {
        let client = NotionClient::new(token, Some(ClientBuilder::new()))
            .map_err(|e| NotionWriterError::ClientError(format!("{:?}", e)))?;
        Ok(Self {
            client,
            database_id,
        })
    }

    /// Inserts a new row into the target Notion database.
    pub async fn write_row(&self, data: NotionRowData) -> Result<(), NotionWriterError> {
        info!(
            symbol = %data.symbol,
            order_id = data.order_id,
            "Writing row to Notion database"
        );

        let mut properties = BTreeMap::new();

        // 1. Symbol (Title column)
        properties.insert(
            "Symbol".to_string(),
            PageProperty::Title {
                id: None,
                title: vec![RichText::Text {
                    text: Text {
                        content: data.symbol,
                        link: None,
                    },
                    annotations: None,
                    plain_text: None,
                    href: None,
                }],
            },
        );

        // 2. Quantity (RichText column)
        properties.insert(
            "Quantity".to_string(),
            PageProperty::RichText {
                id: None,
                rich_text: vec![RichText::Text {
                    text: Text {
                        content: data.quantity,
                        link: None,
                    },
                    annotations: None,
                    plain_text: None,
                    href: None,
                }],
            },
        );

        // 3. Filled Price (RichText column)
        properties.insert(
            "Filled Price".to_string(),
            PageProperty::RichText {
                id: None,
                rich_text: vec![RichText::Text {
                    text: Text {
                        content: data.filled_price,
                        link: None,
                    },
                    annotations: None,
                    plain_text: None,
                    href: None,
                }],
            },
        );

        // 4. Direction (Select column)
        properties.insert(
            "Direction".to_string(),
            PageProperty::Select {
                id: None,
                select: Some(SelectPropertyValue {
                    id: None,
                    name: Some(data.direction),
                    color: None,
                }),
            },
        );

        // 5. Exchange (MultiSelect column)
        properties.insert(
            "Exchange".to_string(),
            PageProperty::MultiSelect {
                id: None,
                multi_select: vec![SelectPropertyValue {
                    id: None,
                    name: Some(data.exchange),
                    color: None,
                }],
            },
        );

        // 6. DataTime (Date column)
        let dt = Utc
            .timestamp_millis_opt(data.time as i64)
            .single()
            .unwrap_or_else(Utc::now);

        properties.insert(
            "DataTime".to_string(),
            PageProperty::Date {
                id: None,
                date: Some(DatePropertyValue {
                    start: Some(DateOrDateTime::DateTime(dt)),
                    end: None,
                    time_zone: None,
                }),
            },
        );

        // 7. Order ID (Number column)
        properties.insert(
            "Order ID".to_string(),
            PageProperty::Number {
                id: None,
                number: Some(Number::from(data.order_id)),
            },
        );

        // 8. Check (Checkbox column)
        properties.insert(
            "Check".to_string(),
            PageProperty::Checkbox {
                id: None,
                checkbox: data.check,
            },
        );

        // Build CreateAPageRequest and create page
        let request = CreateAPageRequestBuilder::default()
            .parent(Parent::DatabaseId {
                database_id: self.database_id.clone(),
            })
            .properties(properties)
            .build()
            .map_err(|e| {
                NotionWriterError::ClientError(format!("Failed to build page request: {:?}", e))
            })?;

        let page = self.client.pages.create_a_page(request).await?;
        info!(page_id = %page.id, "Successfully wrote row to Notion Database");
        Ok(())
    }
}
