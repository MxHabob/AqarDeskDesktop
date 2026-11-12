use crate::{
    application::{ApplicationError, PropertyRepository, require_permission},
    domain::{ActorContext, Permission, Property, PropertyId, PropertyInput},
};

pub struct PropertyCatalog<R> {
    properties: R,
}

impl<R: PropertyRepository> PropertyCatalog<R> {
    pub fn new(properties: R) -> Self {
        Self { properties }
    }

    pub async fn list(&self, actor: ActorContext) -> Result<Vec<Property>, ApplicationError> {
        require_permission(actor, Permission::ViewProperties)?;
        self.properties.list(actor.organization_id()).await
    }

    pub async fn create(
        &self,
        actor: ActorContext,
        input: PropertyInput,
    ) -> Result<Property, ApplicationError> {
        require_permission(actor, Permission::CreateProperties)?;
        let property = Property::create(PropertyId::new(), actor.organization_id(), input)?;
        self.properties.insert(property, actor.user_id()).await
    }

    pub async fn update(
        &self,
        actor: ActorContext,
        property_id: PropertyId,
        input: PropertyInput,
    ) -> Result<Property, ApplicationError> {
        require_permission(actor, Permission::UpdateProperties)?;
        let property = Property::create(property_id, actor.organization_id(), input)?;
        self.properties.update(property, actor.user_id()).await
    }

    pub async fn archive(
        &self,
        actor: ActorContext,
        property_id: PropertyId,
    ) -> Result<(), ApplicationError> {
        require_permission(actor, Permission::ArchiveProperties)?;
        self.properties
            .archive(actor.organization_id(), property_id, actor.user_id())
            .await
    }
}
