use super::{
  album_search_lookup::{
    get_album_search_correlation_id, AlbumSearchLookup, AlbumSearchLookupQuery,
  },
  album_search_lookup_repository::{AggregatedStatus, AlbumSearchLookupRepository},
};
use crate::{
  events::{
    event::{Event, EventPayload, Stream},
    event_publisher::EventPublisher,
  },
  files::file_metadata::file_name::FileName,
};
use anyhow::Result;
use rustis::{bb8::Pool, client::PooledClientManager};
use std::sync::Arc;

pub struct LookupInteractor {
  album_search_lookup_repository: AlbumSearchLookupRepository,
  event_publisher: EventPublisher,
}

impl LookupInteractor {
  pub fn new(redis_connection_pool: Arc<Pool<PooledClientManager>>) -> Self {
    Self {
      album_search_lookup_repository: AlbumSearchLookupRepository {
        redis_connection_pool: Arc::clone(&redis_connection_pool),
      },
      event_publisher: EventPublisher {
        redis_connection_pool: Arc::clone(&redis_connection_pool),
      },
    }
  }

  pub async fn put_album_search_lookup(&self, lookup: &AlbumSearchLookup) -> Result<()> {
    self.album_search_lookup_repository.put(lookup).await
  }

  pub async fn find_album_search_lookup(
    &self,
    query: &AlbumSearchLookupQuery,
  ) -> Result<Option<AlbumSearchLookup>> {
    self.album_search_lookup_repository.find(query).await
  }

  pub async fn get_album_search_lookup(
    &self,
    query: &AlbumSearchLookupQuery,
  ) -> Result<AlbumSearchLookup> {
    self.album_search_lookup_repository.get(query).await
  }

  pub async fn search_album(
    &self,
    artist_name: String,
    album_name: String,
  ) -> Result<AlbumSearchLookup> {
    let query = AlbumSearchLookupQuery::new(album_name, artist_name);
    let lookup = self.album_search_lookup_repository.find(&query).await?;
    match lookup {
      Some(lookup) => Ok(lookup),
      None => {
        let lookup = AlbumSearchLookup::new(query);
        self.put_album_search_lookup(&lookup).await?;
        self
          .event_publisher
          .publish(
            Stream::Lookup,
            EventPayload {
              event: Event::LookupAlbumSearchUpdated {
                lookup: lookup.clone(),
              },
              correlation_id: Some(get_album_search_correlation_id(lookup.query())),
              metadata: None,
            },
          )
          .await?;
        Ok(lookup)
      }
    }
  }

  pub async fn aggregate_statuses(&self) -> Result<Vec<AggregatedStatus>> {
    self
      .album_search_lookup_repository
      .aggregate_statuses()
      .await
  }

  pub async fn find_many_album_search_lookups(
    &self,
    queries: Vec<&AlbumSearchLookupQuery>,
  ) -> Result<Vec<Option<AlbumSearchLookup>>> {
    self.album_search_lookup_repository.find_many(queries).await
  }

  pub async fn find_many_album_search_lookups_by_album_file_name(
    &self,
    album_file_name: &FileName,
  ) -> Result<Vec<AlbumSearchLookup>> {
    self
      .album_search_lookup_repository
      .find_many_by_album_file_name(album_file_name)
      .await
  }
}
