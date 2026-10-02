## ADDED Requirements
### Requirement: Coordinate systems
The mock SHALL answer the coordinate-system methods with canned replies. `GetCoordSystem` SHALL reply `PartCsy`, `GetCsyTransformation` SHALL reply the identity transformation (six zeros), `EnumCoordSystems` SHALL reply an empty list and `GetNamedCsyTransformation` an empty reply. `SetCsyTransformation`, `SaveActiveCoordSystem`, `LoadCoordSystem`, `DeleteCoordSystem` and `SaveNamedCsyTransformation` SHALL be acknowledged without keeping any state.

#### Scenario: Canned coordinate system
- **WHEN** `GetCoordSystem()` is sent
- **THEN** the reply is `CoordSystem(PartCsy)`

