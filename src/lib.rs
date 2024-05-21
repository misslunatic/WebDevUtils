use std::{collections::HashMap, hash::Hash};

use axum::Router;
use log::{info, warn};

pub enum FeatureError<'a> {
    Failure(&'a str),
    DoesNotExist
}

pub trait SiteFeatureStorage {
    fn get_enabled(&self, id: &str) -> bool;
    fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<(), FeatureError>; 
}

pub trait SiteFeature {
    fn get_router(&self) -> Router;
    fn setup(&mut self) -> Result<(), FeatureError>;
    fn shutdown(&mut self) -> Result<(), FeatureError> {
        return Ok(());
    }

    fn get_id(&self) -> &str;
    fn get_subpath(&self) -> &str {
        "/"
    }
    fn get_name(&self) -> &str {
        "Unnamed Feature"
    }
    fn get_description(&self) -> &str {
        "No Description"
    }
}

pub struct SiteFeatureSystem<T: SiteFeatureStorage> {
    storage: T,
    features: HashMap<String, Box<dyn SiteFeature>>
}

impl<T: SiteFeatureStorage> SiteFeatureSystem<T> {
    fn get_all_ids(&self) -> Vec<&str> {
        let mut vec = Vec::new();
        for feature in &self.features {
            vec.push(feature.1.get_id());
        }
        vec
    }

    fn get_router(&self) -> Router {
        let mut router = Router::new();

        for feature in &self.features {
            let feature_router = feature.1.get_router();
            let path = feature.1.get_subpath();
            router = router.nest(path, feature_router);
        }

        router
    }
}

impl<T: SiteFeatureStorage> SiteFeatureStorage for SiteFeatureSystem<T> {
    fn get_enabled(&self, id: &str) -> bool {
        self.storage.get_enabled(id)
    }
    fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<(), FeatureError> {
        let prev_enabled = self.storage.get_enabled(id);
        
        if prev_enabled == enabled {
            return Ok(());
        }

        let feature = self.features.get_mut(id);
        match feature {
            Some(v) => match prev_enabled {
                true => {
                    v.shutdown()?;
                }
                false => {
                    v.setup()?;
                }
            }
            None => {
                return Err(FeatureError::DoesNotExist);
            }
        }

        self.storage.set_enabled(id, enabled);
        Ok(())
    }
}

pub struct SiteFeatureBuilder {
    features: HashMap<String, Box<dyn SiteFeature>>
}

impl SiteFeatureBuilder {
    fn new() -> SiteFeatureBuilder {
        SiteFeatureBuilder {
            features: HashMap::new()
        }
    }

    fn add_feature<F: SiteFeature + 'static>(mut self, feature: F) -> Self {
        let id = feature.get_id().to_string();
        let name: String = feature.get_name().to_string();
        info!("Adding Feature {name} (id of \'{id}\')");

        let prev = self.features.insert(
            id.clone(), 
            Box::new(feature));

        match prev {
            Some(v) => {
                let prev_name = v.get_name();
                warn!("Feature {name} (id of \'{id}\') overrides {prev_name}");
            }
            None => {}
        }

        self
    }

    fn build<T: SiteFeatureStorage>(self, storage: T) -> SiteFeatureSystem::<T> {
        SiteFeatureSystem {
            storage: storage,
            features: self.features
        }
    }
}