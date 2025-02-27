pub use sea_orm_migration::prelude::*;

mod m20220920_000001_create_node_info_table;
mod m20220920_000002_create_group_info_table;
mod m20220920_000003_create_randomness_task_table;
mod m20220920_000004_create_randomness_task_index;
mod m20230612_000005_create_randomness_result_table;
mod m20230612_000006_create_randomness_result_index;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220920_000001_create_node_info_table::Migration),
            Box::new(m20220920_000002_create_group_info_table::Migration),
            Box::new(m20220920_000003_create_randomness_task_table::Migration),
            Box::new(m20220920_000004_create_randomness_task_index::Migration),
            Box::new(m20230612_000005_create_randomness_result_table::Migration),
            Box::new(m20230612_000006_create_randomness_result_index::Migration),
        ]
    }
}

impl Migrator {}
