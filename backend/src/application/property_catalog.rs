use crate::{
    application::{ApplicationError, Page, PageRequest, PropertyRepository, require_permission},
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

    pub async fn list_page(
        &self,
        actor: ActorContext,
        page: PageRequest,
    ) -> Result<Page<Property>, ApplicationError> {
        require_permission(actor, Permission::ViewProperties)?;
        self.properties
            .list_page(actor.organization_id(), page)
            .await
    }

    pub async fn create(
        &self,
        actor: ActorContext,
        mut input: PropertyInput,
    ) -> Result<Property, ApplicationError> {
        require_permission(actor, Permission::CreateProperties)?;
        let id = PropertyId::new();
        input.code = generated_property_code(id);
        let property = Property::create(id, actor.organization_id(), input)?;
        self.properties.insert(property, actor.user_id()).await
    }

    pub async fn update(
        &self,
        actor: ActorContext,
        property_id: PropertyId,
        mut input: PropertyInput,
    ) -> Result<Property, ApplicationError> {
        require_permission(actor, Permission::UpdateProperties)?;
        input.code = generated_property_code(property_id);
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

fn generated_property_code(id: PropertyId) -> String {
    let compact_id = id.as_uuid().simple().to_string();
    format!("PR-{}", &compact_id[..29])
}
