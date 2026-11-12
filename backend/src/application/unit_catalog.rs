use crate::{
    application::{ApplicationError, UnitRepository, require_permission},
    domain::{ActorContext, Permission, PropertyId, PropertyUnit, PropertyUnitInput, UnitId},
};

pub struct UnitCatalog<R> {
    units: R,
}

impl<R: UnitRepository> UnitCatalog<R> {
    pub fn new(units: R) -> Self {
        Self { units }
    }

    pub async fn list(
        &self,
        actor: ActorContext,
        property_id: Option<PropertyId>,
    ) -> Result<Vec<PropertyUnit>, ApplicationError> {
        require_permission(actor, Permission::ViewUnits)?;
        self.units.list(actor.organization_id(), property_id).await
    }

    pub async fn create(
        &self,
        actor: ActorContext,
        input: PropertyUnitInput,
    ) -> Result<PropertyUnit, ApplicationError> {
        require_permission(actor, Permission::CreateUnits)?;
        self.units
            .insert(
                PropertyUnit::create(UnitId::new(), actor.organization_id(), input)?,
                actor.user_id(),
            )
            .await
    }

    pub async fn update(
        &self,
        actor: ActorContext,
        unit_id: UnitId,
        input: PropertyUnitInput,
    ) -> Result<PropertyUnit, ApplicationError> {
        require_permission(actor, Permission::UpdateUnits)?;
        self.units
            .update(
                PropertyUnit::create(unit_id, actor.organization_id(), input)?,
                actor.user_id(),
            )
            .await
    }

    pub async fn archive(
        &self,
        actor: ActorContext,
        unit_id: UnitId,
    ) -> Result<(), ApplicationError> {
        require_permission(actor, Permission::ArchiveUnits)?;
        self.units
            .archive(actor.organization_id(), unit_id, actor.user_id())
            .await
    }
}
