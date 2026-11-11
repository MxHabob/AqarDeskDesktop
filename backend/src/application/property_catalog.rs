use crate::{
    application::{ApplicationError, PropertyRepository, require_write},
    domain::{ActorContext, Property, PropertyId, PropertyInput},
};

pub struct PropertyCatalog<R> {
    properties: R,
}

impl<R: PropertyRepository> PropertyCatalog<R> {
    pub fn new(properties: R) -> Self {
        Self { properties }
    }

    pub async fn list(&self, actor: ActorContext) -> Result<Vec<Property>, ApplicationError> {
        self.properties.list(actor.organization_id()).await
    }

    pub async fn create(
        &self,
        actor: ActorContext,
        input: PropertyInput,
    ) -> Result<Property, ApplicationError> {
        require_write(actor)?;
        let property = Property::create(PropertyId::new(), actor.organization_id(), input)?;
        self.properties.insert(property, actor.user_id()).await
    }

    pub async fn update(
        &self,
        actor: ActorContext,
        property_id: PropertyId,
        input: PropertyInput,
    ) -> Result<Property, ApplicationError> {
        require_write(actor)?;
        let property = Property::create(property_id, actor.organization_id(), input)?;
        self.properties.update(property, actor.user_id()).await
    }

    pub async fn archive(
        &self,
        actor: ActorContext,
        property_id: PropertyId,
    ) -> Result<(), ApplicationError> {
        require_write(actor)?;
        self.properties
            .archive(actor.organization_id(), property_id, actor.user_id())
            .await
    }
}
